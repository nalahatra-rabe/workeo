use std::collections::HashSet;

use serde_json::Value;

use crate::agents::llm::LlmClient;
use crate::models::{Job, MatchScore, Profile};

/// Agent 4 — Matching profil ↔ offres.
///
/// Calcule un score de compatibilité (0-100) et des raisons explicables pour
/// chaque offre. Le LLM est utilisé s'il est disponible ; sinon, un scoring
/// pondéré déterministe et transparent.
pub struct MatcherAgent<'a> {
    llm: &'a LlmClient,
}

impl<'a> MatcherAgent<'a> {
    pub fn new(llm: &'a LlmClient) -> Self {
        MatcherAgent { llm }
    }

    /// Score un lot d'offres pour un profil donné.
    pub async fn score(&self, profile: &Profile, jobs: &[Job]) -> Vec<MatchScore> {
        if jobs.is_empty() {
            return Vec::new();
        }

        // Base heuristique pour toutes les offres (rapide, toujours dispo).
        let mut scores: Vec<MatchScore> =
            jobs.iter().map(|j| score_heuristic(profile, j)).collect();

        // Raffinement LLM pour le top des candidats (limité pour la vitesse).
        if self.llm.is_enabled() {
            let mut order: Vec<usize> = (0..jobs.len()).collect();
            order.sort_by(|&a, &b| scores[b].score.cmp(&scores[a].score));
            let top: Vec<usize> = order.into_iter().take(12).collect();

            if let Some(llm_scores) = self.score_with_llm(profile, jobs, &top).await {
                for s in llm_scores {
                    if let Some(pos) = scores.iter().position(|x| x.job_id == s.job_id) {
                        // Moyenne pondérée 50/50 pour éviter les dérives du LLM.
                        let blended = ((scores[pos].score + s.score) / 2).clamp(0, 100);
                        scores[pos].score = blended;
                        if !s.reasons.is_empty() {
                            scores[pos].reasons = s.reasons;
                        }
                    }
                }
            }
        }

        scores
    }

    async fn score_with_llm(
        &self,
        profile: &Profile,
        jobs: &[Job],
        indices: &[usize],
    ) -> Option<Vec<MatchScore>> {
        let system = "Tu es un expert en recrutement. Évalue la compatibilité entre le profil et \
            chaque offre. Réponds en JSON STRICT: \
            {\"matches\":[{\"job_id\":string,\"score\":number,\"reasons\":[string]}]}. \
            score = entier 0-100. reasons = 2 à 4 phrases courtes et CONCRÈTES en français \
            expliquant pourquoi l'offre correspond (ou non). Sois honnête: pénalise les \
            incompatibilités (séniorité, compétences, contrat, lieu).";

        let jobs_payload: Vec<Value> = indices
            .iter()
            .map(|&i| {
                let j = &jobs[i];
                serde_json::json!({
                    "job_id": j.id,
                    "title": j.title,
                    "company": j.company,
                    "location": j.location,
                    "remote": j.remote,
                    "contract_type": j.contract_type,
                    "seniority": j.seniority,
                    "category": j.category,
                    "skills": j.skills,
                    "summary": j.summary,
                })
            })
            .collect();

        let profile_summary = serde_json::json!({
            "headline": profile.headline,
            "summary": profile.summary,
            "seniority": profile.seniority,
            "years_experience": profile.years_experience,
            "skills": profile.skills,
            "domains": profile.domains,
            "target_roles": profile.target_roles,
            "remote_preference": profile.remote_preference,
            "contract_types": profile.contract_types,
            "salary_min": profile.salary_min,
            "location": profile.location,
        });

        let user = format!(
            "PROFIL:\n{}\n\nOFFRES À ÉVALUER:\n{}",
            serde_json::to_string_pretty(&profile_summary).ok()?,
            serde_json::to_string(&jobs_payload).ok()?
        );

        let v = self.llm.complete_json(system, &user).await?;
        let items = v.get("matches")?.as_array()?;
        Some(
            items
                .iter()
                .filter_map(|i| serde_json::from_value::<MatchScore>(i.clone()).ok())
                .collect(),
        )
    }
}

/// Scoring heuristique transparent et pondéré.
pub fn score_heuristic(profile: &Profile, job: &Job) -> MatchScore {
    let mut score: f64 = 0.0;
    let mut reasons: Vec<String> = Vec::new();

    // 1) Recouvrement de compétences (max 40 pts).
    let job_skills: HashSet<String> =
        job.skills.iter().map(|s| s.to_lowercase()).collect();
    let profile_skills: HashSet<String> =
        profile.skills.iter().map(|s| s.to_lowercase()).collect();
    let matched_skills: Vec<String> = job.skills
        .iter()
        .filter(|s| profile_skills.contains(&s.to_lowercase()))
        .cloned()
        .collect();
    let denom = job_skills.len().min(8).max(1) as f64;
    let skill_ratio = (matched_skills.len() as f64 / denom).min(1.0);
    score += skill_ratio * 40.0;
    if !matched_skills.is_empty() {
        reasons.push(format!(
            "{} compétence(s) en commun : {}",
            matched_skills.len(),
            matched_skills.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        ));
    }

    // 2) Correspondance de catégorie (max 20 pts).
    if let (Some(pc), Some(jc)) = (&profile.category, &job.category) {
        if pc == jc {
            score += 20.0;
            reasons.push(format!("Même domaine métier ({pc})"));
        } else if profile.domains.iter().any(|d| d == jc) {
            score += 12.0;
            reasons.push(format!("Domaine proche ({jc})"));
        }
    }

    // 3) Titre vs rôles ciblés (max 15 pts).
    let target_roles: Vec<String> = profile
        .target_roles
        .iter()
        .map(|r| r.to_lowercase())
        .collect();
    let job_title = job.title.to_lowercase();
    if target_roles.iter().any(|r| job_title.contains(r) || r.contains(&job_title)) {
        score += 15.0;
        reasons.push("Intitulé de poste aligné avec vos objectifs".into());
    } else if target_roles.iter().any(|r| share_word(r, &job_title)) {
        score += 8.0;
        reasons.push("Intitulé de poste proche de vos cibles".into());
    }

    // 4) Séniorité (max 10 pts).
    if let (Some(ps), Some(js)) = (&profile.seniority, &job.seniority) {
        if ps.eq_ignore_ascii_case(js) {
            score += 10.0;
            reasons.push(format!("Niveau de séniorité adapté ({js})"));
        } else if compatible_seniority(ps, js) {
            score += 5.0;
        } else {
            score -= 5.0;
            reasons.push(format!("Niveau demandé ({js}) différent du vôtre ({ps})"));
        }
    }

    // 5) Télétravail (max 8 pts).
    match profile.remote_preference.as_deref() {
        Some("remote") if job.remote => {
            score += 8.0;
            reasons.push("Télétravail, conforme à votre préférence".into());
        }
        Some("on-site") if !job.remote => score += 8.0,
        Some("any") | None => score += 4.0,
        _ => {}
    }

    // 6) Type de contrat (max 7 pts).
    if let Some(ct) = &job.contract_type {
        if profile.contract_types.iter().any(|c| c.eq_ignore_ascii_case(ct)) {
            score += 7.0;
            reasons.push(format!("Type de contrat recherché ({ct})"));
        }
    }

    // 7) Localisation (max 5 pts).
    if let (Some(pl), Some(jl)) = (&profile.location, &job.location) {
        let pl = pl.to_lowercase();
        let jl = jl.to_lowercase();
        if jl.contains(&pl) || pl.contains(&jl) {
            score += 5.0;
            reasons.push(format!("Proche de votre localisation ({jl})"));
        }
    }

    // 8) Salaire (max 5 pts) : bonus si l'offre dépasse l'attente.
    if let (Some(expected), Some(jmin)) = (profile.salary_min, job.salary_min) {
        if jmin >= expected {
            score += 5.0;
            reasons.push("Salaire au niveau de vos attentes".into());
        } else {
            score -= 3.0;
        }
    }

    // 9) Mentions explicites de la catégorie cible dans le titre.
    if let Some(pc) = &profile.category {
        let cat_word = pc.split('/').last().unwrap_or(pc).trim().to_lowercase();
        if !cat_word.is_empty() && job_title.contains(&cat_word) {
            score += 5.0;
        }
    }

    // 10) Fraîcheur de l'offre (max 5 pts).
    if let Some(posted) = job.posted_at {
        let age_days = (chrono::Utc::now() - posted).num_days();
        if age_days <= 7 {
            score += 5.0;
        } else if age_days <= 30 {
            score += 3.0;
        }
    }

    if reasons.is_empty() {
        reasons.push("Aucune correspondance forte détectée — offre à explorer".into());
    }

    MatchScore {
        job_id: job.id.clone(),
        score: score.round().clamp(0.0, 100.0) as i64,
        reasons,
    }
}

fn share_word(a: &str, b: &str) -> bool {
    const STOP: &[&str] = &["de", "du", "des", "la", "le", "les", "et", "en", "a", "the", "of"];
    let wa: Vec<&str> = a.split_whitespace().filter(|w| w.len() > 2 && !STOP.contains(w)).collect();
    b.split_whitespace()
        .filter(|w| w.len() > 2 && !STOP.contains(w))
        .any(|w| wa.contains(&w))
}

fn compatible_seniority(a: &str, b: &str) -> bool {
    let order = ["Stage", "Alternance", "Junior", "Confirmé", "Senior", "Lead", "Expert"];
    match (order.iter().position(|x| x.eq_ignore_ascii_case(a)), order.iter().position(|x| x.eq_ignore_ascii_case(b))) {
        (Some(ia), Some(ib)) => (ia as i32 - ib as i32).abs() <= 1,
        _ => false,
    }
}

/// Rendu lisible d'un score (pour l'UI et les logs).
#[allow(dead_code)]
pub fn score_label(score: i64) -> &'static str {
    match score {
        85..=100 => "Excellent match",
        70..=84 => "Très bon match",
        55..=69 => "Bon match",
        40..=54 => "Match possible",
        _ => "Match faible",
    }
}
