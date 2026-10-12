use chrono::Utc;
use serde_json::Value;

use crate::agents::heuristics;
use crate::agents::llm::LlmClient;
use crate::models::{Job, JobCategorization, RawJob};

/// Agent 3 — Catégorisation & normalisation des offres issues du scraping.
///
/// Nettoie, déduplique, détecte la catégorie métier, les compétences, la
/// séniorité, le type de contrat, le télétravail et le salaire.
pub struct JobCategorizerAgent<'a> {
    llm: &'a LlmClient,
}

impl<'a> JobCategorizerAgent<'a> {
    pub fn new(llm: &'a LlmClient) -> Self {
        JobCategorizerAgent { llm }
    }

    /// Normalise un lot d'offres brutes. Essaie le LLM par lots, sinon heuristique.
    pub async fn categorize_batch(&self, raw: Vec<RawJob>) -> Vec<Job> {
        if raw.is_empty() {
            return Vec::new();
        }

        // Heuristiques d'abord : normalisation + catégorisation de base.
        let mut jobs: Vec<Job> = raw.into_iter().map(normalize_heuristic).collect();

        // Enrichissement LLM par lots (optionnel).
        if self.llm.is_enabled() {
            // Traite par paquets de 8 offres pour limiter la taille du prompt.
            for chunk in jobs.chunks_mut(8) {
                let payload: Vec<Value> = chunk
                    .iter()
                    .map(|j| {
                        serde_json::json!({
                            "external_id": j.external_id,
                            "title": j.title,
                            "company": j.company,
                            "location": j.location,
                            "description": heuristics::summarize(j.description.as_deref().unwrap_or(""), 700),
                            "tags": j.skills,
                        })
                    })
                    .collect();

                if let Some(items) = self.call_llm(&payload).await {
                    apply_llm_results(chunk, items);
                }
            }
        }

        // Déduplication finale par (titre normalisé + entreprise).
        dedup_jobs(jobs)
    }

    async fn call_llm(&self, payload: &[Value]) -> Option<Vec<JobCategorization>> {
        let system = "Tu es un expert en classification d'offres d'emploi. Pour CHAQUE offre \
            fournie, renvoie un JSON STRICT de la forme \
            {\"items\":[{\"external_id\":string,\"category\":string,\"seniority\":string|null,\
            \"skills\":[string],\"contract_type\":string|null,\"remote\":bool,\
            \"salary_min\":number|null,\"salary_max\":number|null,\"salary_currency\":string|null,\
            \"summary\":string}]}. \
            categories possibles: Informatique / Développement, Data / IA, Design / UX, \
            Produit / Gestion de projet, Marketing / Communication, Commerce / Vente, \
            Finance / Comptabilité, Ressources humaines, Santé / Social, Éducation / Formation, \
            Hôtellerie / Restauration / Tourisme, Logistique / Transport, \
            Industrie / Bâtiment / Technique, Sécurité / Accueil, Juridique / Administratif. \
            salary_min/max en valeur annuelle brute, décimales interdites. summary = 1 phrase. \
            Conserve EXACTEMENT les external_id fournis.";

        let user = format!(
            "Offres à classifier (JSON):\n{}",
            serde_json::to_string(payload).ok()?
        );
        let v = self.llm.complete_json(system, &user).await?;
        let items = v.get("items")?.as_array()?;
        let parsed: Vec<JobCategorization> = items
            .iter()
            .filter_map(|i| serde_json::from_value(i.clone()).ok())
            .collect();
        if parsed.is_empty() {
            None
        } else {
            Some(parsed)
        }
    }
}

/// Normalisation purement heuristique d'une offre brute.
pub fn normalize_heuristic(raw: RawJob) -> Job {
    let desc = raw.description.clone().unwrap_or_default();
    let blob = format!("{} {} {}", raw.title, raw.company, desc);

    let category = heuristics::detect_category(&blob);
    let seniority = heuristics::detect_seniority(&format!("{} {}", raw.title, raw.contract_type.clone().unwrap_or_default()))
        .or_else(|| heuristics::detect_seniority(&blob));
    let contract = raw
        .contract_type
        .clone()
        .or_else(|| heuristics::detect_contract(&blob));
    let remote = raw
        .remote
        .unwrap_or_else(|| heuristics::detect_remote(&blob));

    let mut skills = heuristics::detect_skills(&blob);
    // Ajoute les tags de la source qui sont pertinents.
    for tag in &raw.tags {
        let t = heuristics::capitalize_tag(tag);
        if !t.is_empty() && !skills.iter().any(|s| s.eq_ignore_ascii_case(&t)) {
            skills.push(t);
        }
    }
    skills.truncate(15);

    let (salary_min, salary_max, salary_currency) = match raw
        .salary
        .as_deref()
        .and_then(heuristics::parse_salary)
    {
        Some((min, max, cur)) => (Some(min), Some(max), Some(cur)),
        None => (None, None, None),
    };

    let summary = if desc.is_empty() {
        Some(format!("{} chez {}.", raw.title, raw.company))
    } else {
        Some(heuristics::summarize(&desc, 200))
    };

    Job {
        id: uuid::Uuid::new_v4().to_string(),
        source: raw.source,
        external_id: raw.external_id,
        title: clean_title(&raw.title),
        company: clean_company(&raw.company),
        location: raw.location.map(|l| l.trim().to_string()).filter(|l| !l.is_empty()),
        remote,
        description: raw.description,
        url: raw.url,
        summary,
        salary_min,
        salary_max,
        salary_currency,
        contract_type: contract,
        seniority,
        category,
        skills,
        posted_at: raw.posted_at,
        scraped_at: Utc::now(),
    }
}

fn apply_llm_results(jobs: &mut [Job], items: Vec<JobCategorization>) {
    use std::collections::HashMap;
    let map: HashMap<String, JobCategorization> = items
        .into_iter()
        .map(|i| (i.external_id.clone(), i))
        .collect();

    for job in jobs.iter_mut() {
        if let Some(c) = map.get(&job.external_id) {
            if let Some(cat) = &c.category {
                if !cat.trim().is_empty() {
                    job.category = Some(cat.clone());
                }
            }
            if c.seniority.is_some() {
                job.seniority = c.seniority.clone();
            }
            if !c.skills.is_empty() {
                job.skills = c.skills.clone();
            }
            if c.contract_type.is_some() {
                job.contract_type = c.contract_type.clone();
            }
            if let Some(r) = c.remote {
                job.remote = r;
            }
            if c.salary_min.is_some() {
                job.salary_min = c.salary_min;
            }
            if c.salary_max.is_some() {
                job.salary_max = c.salary_max;
            }
            if c.salary_currency.is_some() {
                job.salary_currency = c.salary_currency.clone();
            }
            if let Some(s) = &c.summary {
                if !s.trim().is_empty() {
                    job.summary = Some(s.clone());
                }
            }
            if job.category.is_none() {
                job.category = heuristics::detect_category(&job.title);
            }
        }
    }
}

fn clean_title(t: &str) -> String {
    let t = t.split(['\n', '\r']).next().unwrap_or(t).trim();
    heuristics::summarize(t, 120)
}

fn clean_company(c: &str) -> String {
    heuristics::summarize(c.trim(), 80)
}

/// Supprime les doublons (même titre+boutique, insensible à la casse).
fn dedup_jobs(jobs: Vec<Job>) -> Vec<Job> {
    let mut seen = std::collections::HashSet::new();
    jobs.into_iter()
        .filter(|j| {
            let key = format!(
                "{}-{}",
                j.title.to_lowercase().trim(),
                j.company.to_lowercase().trim()
            );
            seen.insert(key)
        })
        .collect()
}
