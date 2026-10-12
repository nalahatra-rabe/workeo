use crate::agents::heuristics;
use crate::agents::llm::LlmClient;
use crate::models::{CvAnalysis, Education, Experience, Language};

/// Agent 1 — Analyse du CV.
///
/// Transforme un texte de CV brut en données structurées : compétences,
/// séniorité, années d'expérience, formation, langues, expériences, domaines.
pub struct CvAnalyzerAgent<'a> {
    llm: &'a LlmClient,
}

impl<'a> CvAnalyzerAgent<'a> {
    pub fn new(llm: &'a LlmClient) -> Self {
        CvAnalyzerAgent { llm }
    }

    pub async fn analyze(&self, cv_text: &str) -> CvAnalysis {
        if self.llm.is_enabled() {
            if let Some(analysis) = self.analyze_with_llm(cv_text).await {
                return self.reconcile(analysis, cv_text);
            }
        }
        self.analyze_heuristically(cv_text)
    }

    async fn analyze_with_llm(&self, cv_text: &str) -> Option<CvAnalysis> {
        let system = "Tu es un expert RH et parsing de CV. Analyse le CV fourni et réponds \
            STRICTEMENT en JSON valide, sans texte autour, avec cette forme exacte: \
            {\"full_name\":string|null,\"email\":string|null,\"phone\":string|null,\
            \"location\":string|null,\"headline\":string|null,\"summary\":string,\
            \"seniority\":\"Stage|Alternance|Junior|Confirmé|Senior|Lead|Expert\"|null,\
            \"years_experience\":number|null,\
            \"skills\":[string],\"languages\":[{\"name\":string,\"level\":string|null}],\
            \"education\":[{\"degree\":string|null,\"school\":string|null,\"year\":string|null}],\
            \"experiences\":[{\"title\":string|null,\"company\":string|null,\"start\":string|null,\"end\":string|null,\"description\":string|null}],\
            \"domains\":[string],\"strengths\":[string]}. \
            Sois exhaustif sur les skills (techniques ET transverses). Le résumé fait 2 phrases max.";

        let user = format!(
            "Voici le texte du CV (peut contenir du bruit OCR). Extrait toutes les informations.\n\n\
             === DÉBUT CV ===\n{}\n=== FIN CV ===",
            // Tronque pour rester dans la fenêtre de contexte.
            heuristics::summarize(cv_text, 6000)
        );

        let v = self.llm.complete_json(system, &user).await?;
        serde_json::from_value::<CvAnalysis>(v.clone())
            .ok()
            .or_else(|| {
            // Tolérance : certains champs peuvent manquer.
            let mut a: CvAnalysis = serde_json::from_value(v.clone()).unwrap_or_default();
            if a.skills.is_empty() {
                if let Some(s) = v.get("skills").and_then(|x| x.as_array()) {
                    a.skills = s.iter().filter_map(|x| x.as_str().map(String::from)).collect();
                }
            }
            Some(a)
        })
    }

    fn analyze_heuristically(&self, cv_text: &str) -> CvAnalysis {
        let skills = heuristics::detect_skills(cv_text);
        let emails = heuristics::find_emails(cv_text);
        let phones = heuristics::find_phones(cv_text);
        let domains: Vec<String> = heuristics::detect_category(cv_text).into_iter().collect();

        // Titre : première ligne non vide qui ressemble à un intitulé.
        let headline = cv_text
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty() && l.len() < 60 && !l.contains('@'))
            .map(|l| l.to_string());

        // Séniorité : on privilégie les intitulés (titre + 400 premiers caractères).
        let seniority = headline
            .as_deref()
            .and_then(heuristics::detect_seniority)
            .or_else(|| heuristics::detect_seniority(&heuristics::summarize(cv_text, 400)))
            .or_else(|| heuristics::detect_seniority(cv_text));

        let years = heuristics::estimate_years_experience(cv_text);

        // Langues : recherche des noms connus.
        let known_langs = [
            ("Français", "français"), ("Anglais", "anglais"), ("Espagnol", "espagnol"),
            ("Allemand", "allemand"), ("Italien", "italien"), ("Chinois", "chinois"),
            ("Arabe", "arabe"), ("Portugais", "portugais"),
        ];
        let lower = heuristics::normalize(cv_text);
        let languages: Vec<Language> = known_langs
            .iter()
            .filter(|(_, k)| lower.contains(k))
            .map(|(name, _)| Language { name: (*name).into(), level: None })
            .collect();

        // Expériences : détection de lignes contenant une année + un mot de poste.
        let experiences = extract_experiences(cv_text);
        let education = extract_education(cv_text);
        let strengths = heuristics::extract_keywords(cv_text, 6);

        CvAnalysis {
            full_name: None,
            email: emails.first().cloned(),
            phone: phones.first().cloned(),
            location: None,
            headline,
            summary: Some(heuristics::summarize(cv_text, 280)),
            seniority,
            years_experience: years,
            skills,
            languages,
            education,
            experiences,
            domains,
            strengths,
        }
    }

    /// Complète/valide la sortie LLM avec les heuristiques (déduplication, trous).
    fn reconcile(&self, mut a: CvAnalysis, cv_text: &str) -> CvAnalysis {
        a.skills = dedup_preserve(a.skills);
        if a.years_experience.is_none() {
            a.years_experience = heuristics::estimate_years_experience(cv_text);
        }
        if a.seniority.is_none() {
            a.seniority = heuristics::detect_seniority(cv_text);
        }
        if a.email.is_none() {
            a.email = heuristics::find_emails(cv_text).into_iter().next();
        }
        if a.phone.is_none() {
            a.phone = heuristics::find_phones(cv_text).into_iter().next();
        }
        if a.domains.is_empty() {
            if let Some(c) = heuristics::detect_category(cv_text) {
                a.domains.push(c);
            }
        }
        if a.summary.is_none() || a.summary.as_deref().map(|s| s.is_empty()).unwrap_or(true) {
            a.summary = Some(heuristics::summarize(cv_text, 280));
        }
        a
    }
}

fn dedup_preserve(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    items
        .into_iter()
        .filter(|i| !i.trim().is_empty())
        .filter(|i| seen.insert(i.to_lowercase()))
        .collect()
}

fn extract_experiences(text: &str) -> Vec<Experience> {
    // Cherche des motifs "Intitulé — Entreprise (2019 - 2022)" par ligne.
    let re_year = regex::Regex::new(r"(19|20)\d{2}").unwrap();
    let mut out = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.len() < 8 || l.len() > 160 {
            continue;
        }
        if re_year.is_match(l) && !l.to_lowercase().contains("www.") {
            let parts = split_experience_line(l);
            out.push(Experience {
                title: parts.first().map(|s| s.to_string()).filter(|s| !s.is_empty()),
                company: parts.get(1).map(|s| s.to_string()).filter(|s| !s.is_empty()),
                start: re_year.find(l).map(|m| m.as_str().to_string()),
                end: re_year.find_iter(l).last().map(|m| m.as_str().to_string()),
                description: None,
            });
        }
        if out.len() >= 6 {
            break;
        }
    }
    out
}

/// Découpe une ligne d'expérience en (intitulé, entreprise).
/// Privilégie les tirets longs / barres ; ne coupe « Front-end » que par les
/// séparateurs entourés d'espaces.
fn split_experience_line(line: &str) -> Vec<String> {
    for sep in ['—', '–', '|', '·'] {
        if line.contains(sep) {
            return line.split(sep).map(|s| s.trim().to_string()).collect();
        }
    }
    // Tiret simple : uniquement s'il est entouré d'espaces.
    if line.contains(" - ") {
        return line.split(" - ").map(|s| s.trim().to_string()).collect();
    }
    vec![line.trim().to_string()]
}

fn extract_education(text: &str) -> Vec<Education> {
    let lower = heuristics::normalize(text);
    let diplomas = [
        ("Doctorat", "doctorat"), ("Master", "master"), ("Licence", "licence"),
        ("Bachelor", "bachelor"), ("BTS", "bts"), ("DUT", "dut"), ("BUT", "but"),
        ("Baccalauréat", "bac"), ("CAP", "cap"),
    ];
    let mut out = Vec::new();
    for (degree, kw) in diplomas {
        if lower.contains(kw) {
            out.push(Education { degree: Some(degree.into()), school: None, year: None });
        }
    }
    out
}
