//! Sources d'offres concrètes.
//!
//! - **Arbeitnow** et **Remotive** : API JSON publiques (agrégateurs d'offres).
//! - **RemoteOK** : API JSON publique (offres full-remote).
//! - **Démo** : jeu d'offres intégré, fonctionne hors-ligne.
//!
//! Toutes renvoient des [`RawJob`] ; la normalisation est faite ensuite par le
//! `JobCategorizerAgent`.

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use serde_json::Value;

use super::JobSource;
use crate::models::RawJob;

// ─────────────────────────────────────────────────────────────
// Arbeitnow — https://www.arbeitnow.com/api/job-board-api
// ─────────────────────────────────────────────────────────────

pub struct ArbeitnowSource;

#[async_trait]
impl JobSource for ArbeitnowSource {
    fn name(&self) -> &'static str {
        "arbeitnow"
    }
    fn label(&self) -> &'static str {
        "Arbeitnow"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let url = "https://www.arbeitnow.com/api/job-board-api";
        let resp = http.get(url).send().await?.error_for_status()?;
        let json: Value = resp.json().await?;
        let data = json.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();

        let q = query.to_lowercase();
        let loc = location.map(|l| l.to_lowercase());
        let mut out = Vec::new();

        for item in data {
            let title = s(&item, "title");
            let company = s(&item, "company_name");
            if title.is_empty() {
                continue;
            }
            let haystack = format!("{title} {company} {}", s(&item, "description")).to_lowercase();
            if !q.is_empty() && !q.split_whitespace().any(|w| haystack.contains(w)) {
                continue;
            }
            let job_loc = s(&item, "location");
            if let Some(l) = &loc {
                if !l.is_empty() && !job_loc.to_lowercase().contains(l) {
                    continue;
                }
            }
            let tags = item
                .get("tags")
                .and_then(|t| t.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();

            out.push(RawJob {
                source: "arbeitnow".into(),
                external_id: s(&item, "slug"),
                title,
                company,
                location: opt(job_loc),
                description: opt(s(&item, "description")),
                url: opt(s(&item, "url")),
                remote: item.get("remote").and_then(|r| r.as_bool()),
                salary: None,
                contract_type: item
                    .get("job_types")
                    .and_then(|t| t.as_array())
                    .and_then(|a| a.first())
                    .and_then(|x| x.as_str())
                    .map(String::from),
                tags,
                posted_at: item
                    .get("created_at")
                    .and_then(|c| c.as_i64())
                    .and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            });

            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Remotive — https://remotive.com/api/remote-jobs
// ─────────────────────────────────────────────────────────────

pub struct RemotiveSource;

#[async_trait]
impl JobSource for RemotiveSource {
    fn name(&self) -> &'static str {
        "remotive"
    }
    fn label(&self) -> &'static str {
        "Remotive"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let mut url = format!("https://remotive.com/api/remote-jobs?limit={limit}");
        if !query.is_empty() {
            url.push_str(&format!("&search={}", urlencode(query)));
        }
        let resp = http.get(&url).send().await?.error_for_status()?;
        let json: Value = resp.json().await?;
        let jobs = json.get("jobs").and_then(|d| d.as_array()).cloned().unwrap_or_default();

        let loc = location.map(|l| l.to_lowercase());
        let mut out = Vec::new();
        for item in jobs {
            let title = s(&item, "title");
            if title.is_empty() {
                continue;
            }
            let job_loc = s(&item, "candidate_required_location");
            if let Some(l) = &loc {
                if !l.is_empty() && !job_loc.to_lowercase().contains(l) {
                    continue;
                }
            }
            out.push(RawJob {
                source: "remotive".into(),
                external_id: item.get("id").map(|v| v.to_string()).unwrap_or_else(|| title.clone()),
                title,
                company: s(&item, "company_name"),
                location: opt(job_loc),
                description: opt(strip_html(&s(&item, "description"))),
                url: opt(s(&item, "url")),
                remote: Some(true),
                salary: opt(s(&item, "salary")),
                contract_type: opt(s(&item, "job_type")),
                tags: item
                    .get("tags")
                    .and_then(|t| t.as_array())
                    .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                    .unwrap_or_default(),
                posted_at: s(&item, "publication_date")
                    .parse::<DateTime<Utc>>()
                    .ok(),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// RemoteOK — https://remoteok.com/api
// ─────────────────────────────────────────────────────────────

pub struct RemoteOkSource;

#[async_trait]
impl JobSource for RemoteOkSource {
    fn name(&self) -> &'static str {
        "remoteok"
    }
    fn label(&self) -> &'static str {
        "RemoteOK"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        _location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let resp = http
            .get("https://remoteok.com/api")
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?;
        let json: Value = resp.json().await?;
        let arr = json.as_array().cloned().unwrap_or_default();

        let q = query.to_lowercase();
        let mut out = Vec::new();
        for item in arr {
            // Le premier élément est une notice légale sans "position".
            let title = s(&item, "position");
            if title.is_empty() {
                continue;
            }
            let tags: Vec<String> = item
                .get("tags")
                .and_then(|t| t.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let haystack = format!("{title} {} {}", s(&item, "company"), tags.join(" ")).to_lowercase();
            if !q.is_empty() && !q.split_whitespace().any(|w| haystack.contains(w)) {
                continue;
            }
            let salary = match (
                item.get("salary_min").and_then(|v| v.as_i64()),
                item.get("salary_max").and_then(|v| v.as_i64()),
            ) {
                (Some(mn), Some(mx)) if mn > 0 => Some(format!("{mn}-{mx} USD")),
                _ => None,
            };
            out.push(RawJob {
                source: "remoteok".into(),
                external_id: item
                    .get("id")
                    .map(|v| v.to_string())
                    .or_else(|| opt(s(&item, "slug")))
                    .unwrap_or_else(|| title.clone()),
                title,
                company: s(&item, "company"),
                location: opt(s(&item, "location")).or(Some("Remote".into())),
                description: opt(strip_html(&s(&item, "description"))),
                url: opt(s(&item, "url")),
                remote: Some(true),
                salary,
                contract_type: None,
                tags,
                posted_at: s(&item, "date").parse::<DateTime<Utc>>().ok(),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Démo — jeu d'offres intégré (hors-ligne)
// ─────────────────────────────────────────────────────────────

pub struct DemoSource;

#[async_trait]
impl JobSource for DemoSource {
    fn name(&self) -> &'static str {
        "demo"
    }
    fn label(&self) -> &'static str {
        "Démo (hors-ligne)"
    }

    async fn fetch(
        &self,
        _http: &Client,
        _query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        // La source démo renvoie un panel varié : le classement par pertinence
        // (MatcherAgent) fait ensuite le tri. On ne filtre que par lieu.
        let samples = demo_samples();
        let loc = location.map(|l| l.to_lowercase());

        let out = samples
            .into_iter()
            .filter(|j| match &loc {
                Some(l) if !l.is_empty() => j
                    .location
                    .as_deref()
                    .map(|jl| jl.to_lowercase().contains(l) || jl.to_lowercase().contains("remote"))
                    .unwrap_or(true),
                _ => true,
            })
            .take(limit)
            .collect();
        Ok(out)
    }
}

fn demo_samples() -> Vec<RawJob> {
    let mk = |id: &str, title: &str, company: &str, location: &str, remote: bool, salary: Option<&str>, ct: &str, tags: &[&str], desc: &str| RawJob {
        source: "demo".to_string(),
        external_id: id.to_string(),
        title: title.to_string(),
        company: company.to_string(),
        location: Some(location.to_string()),
        description: Some(desc.to_string()),
        url: Some(format!("https://exemple.fr/offres/{id}")),
        remote: Some(remote),
        salary: salary.map(String::from),
        contract_type: Some(ct.to_string()),
        tags: tags.iter().map(|s| s.to_string()).collect(),
        posted_at: Some(Utc::now() - chrono::Duration::days((id.len() as i64) % 20)),
    };

    vec![
        mk("d1", "Développeur Fullstack React/Node.js", "Novatech", "Paris", true, Some("45 000 - 55 000 €"), "CDI",
            &["react", "node.js", "typescript", "postgresql", "docker"],
            "Nous recherchons un développeur fullstack pour concevoir des applications web modernes. Stack: React, TypeScript, Node.js, PostgreSQL, Docker. Télétravail 3j/semaine. 3 ans d'expérience souhaités."),
        mk("d2", "Développeur Rust — Systèmes distribués", "CloudForge", "Lyon", true, Some("55k€"), "CDI",
            &["rust", "kubernetes", "grpc", "tokio"],
            "Rejoignez une équipe passionnée pour construire des services backend à haute performance en Rust (Tokio, gRPC). Environnement Kubernetes, forte culture de l'ingénierie."),
        mk("d3", "Alternance Développeur Web Front-end", "Studio Pixel", "Bordeaux", false, Some("1 000 €/mois"), "Alternance",
            &["javascript", "vue", "css", "html"],
            "Dans le cadre d'un BUT Informatique, venez participer à la création d'interfaces web. Stack Vue.js, CSS, intégration Figma. Alternance 12 mois."),
        mk("d4", "Stage Data Analyst", "DataMind", "Nantes", true, None, "Stage",
            &["python", "sql", "pandas", "power bi"],
            "Stage de 6 mois en analyse de données: extraction SQL, nettoyage Python/Pandas, dashboards Power BI. Idéal pour un Master en data science."),
        mk("d5", "Data Scientist Machine Learning", "DeepCore AI", "Remote", true, Some("60 000 - 75 000 €"), "CDI",
            &["python", "machine learning", "pytorch", "nlp", "mlops"],
            "Nous cherchons un data scientist senior pour concevoir des modèles de NLP en production (PyTorch, MLOps). Full remote possible."),
        mk("d6", "UX/UI Designer", "Fintech One", "Paris", false, Some("42k€"), "CDI",
            &["figma", "ux", "ui", "design system"],
            "Concevez l'expérience de notre application bancaire: recherche utilisateur, wireframes, design system Figma, tests d'usabilité."),
        mk("d7", "Chef de projet digital", "Agence Flux", "Marseille", false, Some("40 000 €"), "CDI",
            &["gestion de projet", "agile", "scrum", "jira"],
            "Pilotage de projets web en méthode agile (Scrum). Coordination des équipes dev et design, suivi des plannings et budgets."),
        mk("d8", "Comptable général", "Cabinet Aurore", "Toulouse", false, Some("32 000 €"), "CDI",
            &["comptabilité", "sage", "excel", "fiscalité"],
            "Tenue de la comptabilité générale, déclarations fiscales, révision des comptes. Maîtrise de Sage et Excel indispensable."),
        mk("d9", "Commercial B2B", "SalesBoost", "Lille", false, Some("35 000 € + variable"), "CDI",
            &["vente", "négociation", "crm", "business development"],
            "Développement d'un portefeuille clients B2B, prospection et négociation. CRM Salesforce. Statut cadre."),
        mk("d10", "Aide-soignant (H/F)", "Clinique du Parc", "Grenoble", false, None, "CDI",
            &["soins", "aide à la personne"],
            "Accompagnement des patients au quotidien, soins d'hygiène et confort, collaboration avec l'équipe soignante. Diplôme d'État requis."),
        mk("d11", "Développeur Backend Java Spring", "Banque Nationale", "Paris", true, Some("50 000 - 60 000 €"), "CDI",
            &["java", "spring", "sql", "kubernetes", "microservices"],
            "Développement de services bancaires en Java/Spring Boot, architecture microservices, CI/CD. 5 ans d'expérience minimum."),
        mk("d12", "Alternance Marketing Digital", "GreenLife", "Rennes", true, Some("900 €/mois"), "Alternance",
            &["marketing", "seo", "social media", "communication"],
            "Alternance en marketing digital: animation des réseaux sociaux, rédaction de contenu, optimisation SEO. Master Marketing."),
        mk("d13", "Technicien de maintenance industrielle", "MécaPro", "Saint-Étienne", false, Some("30 000 €"), "CDI",
            &["maintenance", "mécanique", "électricité"],
            "Maintenance préventive et curative de lignes de production. Diagnostique électrique et mécanique. Habilitation électrique appréciée."),
        mk("d14", "Product Owner", "SaaS Scale", "Remote", true, Some("55k€"), "CDI",
            &["product management", "agile", "scrum", "jira"],
            "Vous priorisez le backlog, animez les rituels agiles et collaborez avec les devs. Expérience en SaaS B2B requise. Full remote."),
        mk("d15", "Ingénieur DevOps", "InfraCloud", "Remote", true, Some("58 000 - 70 000 €"), "CDI",
            &["docker", "kubernetes", "terraform", "aws", "ci/cd", "linux"],
            "Automatisation de l'infrastructure AWS avec Terraform, pipelines CI/CD, orchestration Kubernetes. Culture DevOps."),
    ]
}

// ─── Helpers ────────────────────────────────────────────────

fn s(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").trim().to_string()
}

fn opt(s: String) -> Option<String> {
    if s.trim().is_empty() { None } else { Some(s) }
}

/// Retire rapidement les balises HTML d'une description.
pub fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Normalise les entités courantes et les espaces.
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u32 as u8),
        })
        .collect()
}
