use chrono::Utc;

use crate::agents::cv_analyzer::CvAnalyzerAgent;
use crate::agents::job_categorizer::JobCategorizerAgent;
use crate::agents::matcher::MatcherAgent;
use crate::agents::profile_categorizer::ProfileCategorizerAgent;
use crate::db;
use crate::models::{Job, Profile, ScoredJob, SearchRequest, SearchResult, SearchRun};
use crate::scraper;
use crate::state::AppState;

/// Agent 5 — Orchestrateur.
///
/// Coordonne l'ensemble du pipeline : recherche web multi-sources →
/// catégorisation des offres → scoring de compatibilité → persistance.
pub struct SearchOrchestrator;

/// Paramètres de tuning du pipeline.
const JOBS_PER_SOURCE: usize = 30;
const CANDIDATE_POOL: i64 = 250;
const RESULT_LIMIT: usize = 60;

impl SearchOrchestrator {
    /// Lance une recherche complète pour un profil.
    pub async fn run(state: &AppState, req: &SearchRequest) -> anyhow::Result<SearchResult> {
        let profile = db::get_profile(&state.pool, &req.profile_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("profil introuvable"))?;

        // 1) Déterminer la requête et les sources.
        let query = req
            .query
            .clone()
            .filter(|q| !q.trim().is_empty())
            .or_else(|| derive_query(&profile))
            .unwrap_or_default();

        let location = req
            .location
            .clone()
            .or_else(|| profile.location.clone());

        if let (Some(lat), Some(lon)) = (req.latitude, req.longitude) {
            tracing::info!(lat, lon, "recherche géolocalisée");
        }

        let sources = req
            .sources
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| state.config.scrape_sources.clone());

        // 2) Créer le run de recherche.
        let run = SearchRun {
            id: uuid::Uuid::new_v4().to_string(),
            profile_id: profile.id.clone(),
            query: Some(query.clone()),
            location: location.clone(),
            sources: sources.clone(),
            status: "running".into(),
            jobs_found: 0,
            jobs_new: 0,
            started_at: Utc::now(),
            finished_at: None,
        };
        db::insert_search_run(&state.pool, &run).await?;

        // 3) Scraping parallèle.
        let outcome = scraper::fetch_all(
            &state.http,
            &sources,
            &query,
            location.as_deref(),
            JOBS_PER_SOURCE,
        )
        .await;
        let source_errors = outcome.errors;
        let raw_jobs = outcome.jobs;

        // 4) Catégorisation + persistance des offres.
        let categorizer = JobCategorizerAgent::new(&state.llm);
        let categorized = categorizer.categorize_batch(raw_jobs).await;

        let mut jobs_new = 0i64;
        for job in &categorized {
            match db::upsert_job(&state.pool, job).await {
                Ok(true) => jobs_new += 1,
                Ok(false) => {}
                Err(e) => tracing::warn!("upsert job: {e}"),
            }
        }

        // 5) Constituer le pool de candidats (offres récentes correspondant aux filtres).
        let candidates = load_candidates(state, req).await.unwrap_or_default();

        // 6) Scoring.
        let matcher = MatcherAgent::new(&state.llm);
        let scores = matcher.score(&profile, &candidates).await;

        // 7) Persister les matchs + construire le résultat.
        let mut results: Vec<ScoredJob> = Vec::new();
        for score in &scores {
            let Some(job) = candidates.iter().find(|j| j.id == score.job_id) else {
                continue;
            };
            if let Err(e) = db::upsert_match(
                &state.pool,
                &profile.id,
                &job.id,
                score.score,
                &score.reasons,
            )
            .await
            {
                tracing::warn!("upsert match: {e}");
            }
            results.push(ScoredJob {
                job: job.clone(),
                score: score.score,
                reasons: score.reasons.clone(),
            });
        }
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(RESULT_LIMIT);

        let total = results.len() as i64;

        // 8) Clôturer le run.
        db::finish_search_run(&state.pool, &run.id, "done", categorized.len() as i64, jobs_new)
            .await?;

        let mut run = run;
        run.status = "done".into();
        run.jobs_found = categorized.len() as i64;
        run.jobs_new = jobs_new;
        run.finished_at = Some(Utc::now());

        Ok(SearchResult {
            run,
            total,
            results,
            source_errors,
        })
    }

    /// Analyse un CV : `CvAnalyzerAgent` + `ProfileCategorizerAgent`.
    pub async fn analyze_profile(
        state: &AppState,
        profile: &mut Profile,
    ) -> anyhow::Result<()> {
        let text = profile.cv_text.clone().unwrap_or_default();
        if !text.trim().is_empty() {
            let analyzer = CvAnalyzerAgent::new(&state.llm);
            let analysis = analyzer.analyze(&text).await;
            apply_analysis(profile, analysis);
        }
        let categorizer = ProfileCategorizerAgent::new(&state.llm);
        let cat = categorizer.categorize(profile).await;
        profile.category = cat.primary_category.clone();
        if !cat.categories.is_empty() {
            profile.domains = cat.categories.clone();
        }
        if !cat.search_keywords.is_empty() {
            profile.search_keywords = cat.search_keywords.clone();
        }
        if !cat.suggested_titles.is_empty() {
            profile.target_roles = cat.suggested_titles.clone();
        }
        if profile.remote_preference.is_none() {
            profile.remote_preference = cat.remote_preference.clone();
        }
        profile.updated_at = Utc::now();
        db::update_profile(&state.pool, profile).await?;
        Ok(())
    }
}

/// Applique une analyse de CV sur un profil, sans écraser les infos fournies.
fn apply_analysis(profile: &mut Profile, a: crate::models::CvAnalysis) {
    if profile.full_name.trim().is_empty() {
        if let Some(n) = a.full_name {
            profile.full_name = n;
        }
    }
    if profile.email.is_none() {
        profile.email = a.email;
    }
    if profile.phone.is_none() {
        profile.phone = a.phone;
    }
    if profile.location.is_none() {
        profile.location = a.location;
    }
    if profile.headline.is_none() {
        profile.headline = a.headline;
    }
    if profile.summary.is_none() {
        profile.summary = a.summary;
    }
    if profile.seniority.is_none() {
        profile.seniority = a.seniority;
    }
    if profile.years_experience.is_none() {
        profile.years_experience = a.years_experience;
    }
    if !a.skills.is_empty() {
        let mut set = profile.skills.clone();
        for s in a.skills {
            if !set.iter().any(|x| x.eq_ignore_ascii_case(&s)) {
                set.push(s);
            }
        }
        profile.skills = set;
    }
    if profile.languages.is_empty() {
        profile.languages = a.languages;
    }
    if profile.education.is_empty() {
        profile.education = a.education;
    }
    if profile.experiences.is_empty() {
        profile.experiences = a.experiences;
    }
    if profile.domains.is_empty() {
        profile.domains = a.domains;
    }
    profile.updated_at = Utc::now();
}

/// Charge un pool d'offres candidates depuis la base, filtré par la requête.
async fn load_candidates(state: &AppState, req: &SearchRequest) -> anyhow::Result<Vec<Job>> {
    let remote_only = req.remote_only.unwrap_or(false);
    let contract = req.contract_type.clone();
    let location = req.location.clone();

    let rows = sqlx::query(
        "SELECT * FROM jobs ORDER BY scraped_at DESC LIMIT ?1",
    )
    .bind(CANDIDATE_POOL)
    .fetch_all(&state.pool)
    .await?;

    let mut jobs: Vec<Job> = rows.iter().map(db::row_to_job).collect();

    if remote_only {
        jobs.retain(|j| j.remote);
    }
    if let Some(ct) = &contract {
        let ct = ct.to_lowercase();
        jobs.retain(|j| {
            j.contract_type
                .as_deref()
                .map(|c| c.to_lowercase().contains(&ct))
                .unwrap_or(false)
        });
    }
    if let Some(loc) = &location {
        let loc = loc.to_lowercase();
        if !loc.is_empty() {
            jobs.retain(|j| {
                j.remote
                    || j.location
                        .as_deref()
                        .map(|l| l.to_lowercase().contains(&loc))
                        .unwrap_or(false)
            });
        }
    }

    Ok(jobs)
}

/// Dérive une requête de recherche à partir du profil.
fn derive_query(profile: &Profile) -> Option<String> {
    if let Some(first) = profile.search_keywords.first() {
        return Some(first.clone());
    }
    if let Some(first) = profile.target_roles.first() {
        return Some(first.clone());
    }
    if let Some(h) = &profile.headline {
        return Some(h.clone());
    }
    profile.skills.first().cloned()
}
