//! Heuristiques partagées : dictionnaires, détection de catégories, parsing.
//!
//! Ces fonctions servent de **fallback** lorsque le LLM est désactivé ou
//! injoignable, et de **post-traitement** pour normaliser la sortie du LLM.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::BTreeSet;

// ─── Dictionnaire de compétences ────────────────────────────

/// Liste de compétences reconnues (techniques + transverses + métiers).
pub static SKILLS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        // Langages & dev
        "rust", "python", "javascript", "typescript", "java", "c++", "c#", "go", "php", "ruby",
        "kotlin", "swift", "scala", "dart", "sql", "r", "matlab", "bash", "html", "css", "sass",
        // Frameworks / libs
        "react", "vue", "angular", "svelte", "next.js", "node.js", "django", "flask", "fastapi",
        "spring", "laravel", "rails", "express", "nuxt", "flutter", "react native", ".net",
        "symfony", "tailwind", "bootstrap", "jquery",
        // Data / IA
        "machine learning", "deep learning", "nlp", "tensorflow", "pytorch", "scikit-learn",
        "pandas", "numpy", "spark", "hadoop", "airflow", "dbt", "power bi", "tableau", "etl",
        "data analysis", "data science", "big data", "statistiques",
        // Cloud / infra / devops
        "docker", "kubernetes", "terraform", "ansible", "aws", "azure", "gcp", "ci/cd", "jenkins",
        "gitlab ci", "github actions", "linux", "git", "nginx", "postgresql", "mysql", "mongodb",
        "redis", "elasticsearch", "rabbitmq", "kafka", "graphql", "rest api", "microservices",
        "serverless", "devops",
        // Gestion / produit / design
        "gestion de projet", "agile", "scrum", "kanban", "product management", "jira", "confluence",
        "figma", "ux", "ui", "design", "photoshop", "illustrator", "adobe xd", "seo", "sea",
        "marketing", "growth", "communication", "copywriting", "community management",
        // Comptabilité / finance / RH / juridique
        "comptabilité", "audit", "contrôle de gestion", "finance", "fiscalité", "paie", "sage",
        "excel", "ressources humaines", "recrutement", "droit", "juridique",
        // Commerce / relation client / terrain
        "vente", "négociation", "relation client", "crm", "salesforce", "hubspot", "business development",
        "support client", "call center", "logistique", "achats", "supply chain",
        // Santé / social / éducation / industrie
        "soins", "aide à la personne", "animation", "enseignement", "formation", "garderie",
        "sécurité", "maintenance", "électricité", "plomberie", "mécanique", "bâtiment", "haccp",
        "restauration", "cuisine", "accueil", "hôtellerie",
        // Langues / soft skills
        "anglais", "français", "espagnol", "allemand", "italien", "chinois", "arabe",
        "travail d'équipe", "autonomie", "rigueur", "adaptabilité", "esprit d'analyse",
        "organisation", "savoir-être", "management", "leadership",
    ]
});

// ─── Dictionnaire de catégories métier ──────────────────────

/// Associe une catégorie lisible à une liste de mots-clés déclencheurs.
pub static CATEGORIES: Lazy<Vec<(&'static str, Vec<&'static str>)>> = Lazy::new(|| {
    vec![
        (
            "Informatique / Développement",
            vec![
                "développeur", "developer", "software", "engineer", "ingénieur logiciel", "fullstack",
                "full-stack", "front-end", "frontend", "back-end", "backend", "web", "mobile",
                "rust", "python", "java", "javascript", "typescript", "react", "node", "devops",
                "sre", "programmeur", "intégrateur", "architecte logiciel",
            ],
        ),
        (
            "Data / IA",
            vec![
                "data", "données", "machine learning", "ia", "intelligence artificielle", "analyst",
                "scientist", "bi", "business intelligence", "etl", "big data", "nlp", "mlops",
            ],
        ),
        (
            "Design / UX",
            vec!["designer", "ux", "ui", "graphiste", "graphic", "figma", "maquette", "ergonomie", "direction artistique"],
        ),
        (
            "Produit / Gestion de projet",
            vec!["product", "produit", "chef de projet", "project manager", "scrum master", "product owner", "pmo", "agile"],
        ),
        (
            "Marketing / Communication",
            vec![
                "marketing", "communication", "seo", "sea", "growth", "social media", "community",
                "content", "rédacteur", "copywriter", "brand", "publicité", "événementiel",
            ],
        ),
        (
            "Commerce / Vente",
            vec![
                "commercial", "sales", "vente", "business developer", "account manager", "vendeur",
                "conseiller de vente", "chargé d'affaires", "négociation", "crm",
            ],
        ),
        (
            "Finance / Comptabilité",
            vec![
                "comptable", "comptabilité", "audit", "finance", "contrôle de gestion", "fiscal",
                "paie", "trésorerie", "banque", "assurance", "analyste financier",
            ],
        ),
        (
            "Ressources humaines",
            vec!["ressources humaines", "rh", "recrutement", "recruteur", "talent", "formation professionnelle"],
        ),
        (
            "Santé / Social",
            vec![
                "infirmier", "aide-soignant", "santé", "médical", "pharmacie", "kiné", "social",
                "éducateur", "aide à la personne", "auxiliaire", "sage-femme", "soin",
            ],
        ),
        (
            "Éducation / Formation",
            vec!["professeur", "enseignant", "formateur", "éducation", "scolaire", "animateur", "moniteur"],
        ),
        (
            "Hôtellerie / Restauration / Tourisme",
            vec!["cuisinier", "serveur", "restauration", "hôtel", "réceptionniste", "barista", "tourisme", "agence de voyage"],
        ),
        (
            "Logistique / Transport",
            vec!["logistique", "transport", "chauffeur", "magasinier", "approvisionnement", "supply chain", "achats", "entrepôt"],
        ),
        (
            "Industrie / Bâtiment / Technique",
            vec![
                "technicien", "maintenance", "électricien", "mécanicien", "bâtiment", "chantier",
                "ingénieur", "production", "usine", "qualité", "hse", "btp", "artisan",
            ],
        ),
        (
            "Sécurité / Accueil",
            vec!["agent de sécurité", "vigile", "accueil", "standard", "surveillance", "sécurité privée"],
        ),
        (
            "Juridique / Administratif",
            vec!["juridique", "avocat", "assistant administratif", "secrétaire", "back office", "administration", "notaire"],
        ),
    ]
});

/// Junior / senior detection keywords.
static SENIOR_KEYWORDS: Lazy<Vec<(&'static str, &'static str)>> = Lazy::new(|| {
    vec![
        ("stage", "Stage"),
        ("stagiaire", "Stage"),
        ("alternance", "Alternance"),
        ("alternant", "Alternance"),
        ("apprentissage", "Alternance"),
        ("apprenti", "Alternance"),
        ("junior", "Junior"),
        ("débutant", "Junior"),
        ("debutant", "Junior"),
        ("entry level", "Junior"),
        ("stagiaire", "Stage"),
        ("confirmé", "Confirmé"),
        ("confirme", "Confirmé"),
        ("mid-level", "Confirmé"),
        ("senior", "Senior"),
        ("sénior", "Senior"),
        ("lead", "Lead"),
        ("principal", "Lead"),
        ("expert", "Expert"),
        ("manager", "Manager"),
        ("directeur", "Directeur"),
        ("head of", "Directeur"),
        ("chief", "Directeur"),
    ]
});

static CONTRACT_KEYWORDS: Lazy<Vec<(&'static str, &'static str)>> = Lazy::new(|| {
    vec![
        ("cdi", "CDI"),
        ("cdd", "CDD"),
        ("stage", "Stage"),
        ("stagiaire", "Stage"),
        ("alternance", "Alternance"),
        ("apprentissage", "Alternance"),
        ("professionnalisation", "Alternance"),
        ("intérim", "Intérim"),
        ("interim", "Intérim"),
        ("freelance", "Freelance"),
        ("indépendant", "Freelance"),
        ("temps partiel", "Temps partiel"),
        ("part-time", "Temps partiel"),
        ("contract", "CDD"),
    ]
});

// ─── Helpers texte ──────────────────────────────────────────

pub fn normalize(text: &str) -> String {
    text.to_lowercase()
}

/// Détecte les compétences présentes dans un texte (matching sur limites de mot).
pub fn detect_skills(text: &str) -> Vec<String> {
    let lower = normalize(text);
    let mut found: BTreeSet<String> = BTreeSet::new();
    for skill in SKILLS.iter() {
        let s = skill.to_lowercase();
        if s.len() >= 2 && contains_bounded(&lower, &s) {
            found.insert(capitalize_skill(skill));
        }
    }
    found.into_iter().collect()
}

/// Vrai si `needle` apparaît dans `hay` en étant délimité par des caractères
/// non alphanumériques (évite « java » ⊂ « javascript », « go » ⊂ « google »…).
pub fn contains_bounded(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let bytes = hay.as_bytes();
    let n = needle.as_bytes();
    let is_word = |c: u8| c.is_ascii_alphanumeric();
    let mut start = 0usize;
    while let Some(pos) = hay[start..].find(needle) {
        let abs = start + pos;
        let before_ok = abs == 0 || !is_word(bytes[abs - 1]);
        let end = abs + n.len();
        let after_ok = end >= bytes.len() || !is_word(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
        if start >= hay.len() {
            break;
        }
    }
    false
}

/// Normalise un tag de source en libellé lisible (ex: "full-stack" -> "Full-stack").
pub fn capitalize_tag(tag: &str) -> String {
    let t = tag.trim();
    if t.is_empty() || t.len() > 40 {
        return String::new();
    }
    // Ignore les tags purement numériques ou trop génériques.
    if t.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '_') {
        return String::new();
    }
    let t = t.replace(['_', '-'], " ");
    let mut c = t.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn capitalize_skill(s: &str) -> String {
    // Garde les acronymes connus en majuscules.
    const ACRONYMS: &[&str] = &[
        "aws", "gcp", "sql", "nlp", "etl", "bi", "ui", "ux", "crm", "seo", "sea", "ci/cd", "rh",
        "pmo", "hse", "btp",
    ];
    if ACRONYMS.contains(&s.to_lowercase().as_str()) {
        return s.to_uppercase();
    }
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Devine la catégorie métier la plus probable d'un texte.
pub fn detect_category(text: &str) -> Option<String> {
    let lower = normalize(text);
    let mut best: Option<(&str, usize)> = None;
    for (cat, keywords) in CATEGORIES.iter() {
        let mut score = 0usize;
        for kw in keywords {
            if contains_bounded(&lower, &kw.to_lowercase()) {
                score += 1;
            }
        }
        if score > 0 {
            match best {
                Some((_, s)) if s >= score => {}
                _ => best = Some((cat, score)),
            }
        }
    }
    best.map(|(c, _)| c.to_string())
}

/// Détecte le niveau de séniorité (renvoie la 1re correspondance pertinente).
pub fn detect_seniority(text: &str) -> Option<String> {
    let lower = normalize(text);
    for (kw, level) in SENIOR_KEYWORDS.iter() {
        if lower.contains(kw) {
            return Some((*level).to_string());
        }
    }
    None
}

/// Détecte le type de contrat.
pub fn detect_contract(text: &str) -> Option<String> {
    let lower = normalize(text);
    for (kw, ct) in CONTRACT_KEYWORDS.iter() {
        if lower.contains(kw) {
            return Some((*ct).to_string());
        }
    }
    None
}

/// Détecte le télétravail à partir d'un texte et d'un lieu.
pub fn detect_remote(text: &str) -> bool {
    let lower = normalize(text);
    ["télétravail", "teletravail", "remote", "à distance", "a distance", "full remote",
     "home office", "homeoffice", "100% remote", "hybride", "hybrid", "distanciel"]
        .iter()
        .any(|kw| lower.contains(kw))
}

/// Estime les années d'expérience à partir de dates trouvées (ex: 2018 - 2021).
pub fn estimate_years_experience(text: &str) -> Option<f64> {
    let re = Regex::new(r"(19|20)\d{2}").unwrap();
    let years: Vec<i32> = re
        .find_iter(text)
        .filter_map(|m| m.as_str().parse().ok())
        .filter(|y| (1970..=2100).contains(y))
        .collect();
    if years.is_empty() {
        return None;
    }
    let min = *years.iter().min().unwrap();
    let max = *years.iter().max().unwrap();
    if max <= min {
        return None;
    }
    Some((max - min) as f64)
}

/// Extrait un montant de salaire (approx.) depuis un texte.
/// Renvoie (min, max, devise) en valeur annuelle brute quand c'est possible.
pub fn parse_salary(text: &str) -> Option<(i64, i64, String)> {
    let lower = normalize(text).replace([' ', '\u{a0}'], "");
    // Formats: "45k€", "45 000 €", "€45000", "$120k", "45000-55000"
    let re = Regex::new(r"(\d{2,3})(k)?(?:-|à|a|–|—)?(\d{2,3})?(k)?").unwrap();
    let mut amounts: Vec<i64> = Vec::new();
    for cap in re.captures_iter(&lower) {
        let a: i64 = cap[1].parse().unwrap_or(0);
        let a_k = cap.get(2).is_some();
        let mut val = if a_k || a < 1000 { a * 1000 } else { a };
        if let Some(b) = cap.get(3) {
            let b: i64 = b.as_str().parse().unwrap_or(0);
            let b_k = cap.get(4).is_some();
            let bval = if b_k || b < 1000 { b * 1000 } else { b };
            if bval > val {
                val = bval;
            }
        }
        // Filtre les valeurs plausibles (annuel 12k..500k).
        if (10000..=500000).contains(&val) {
            amounts.push(val);
        }
    }
    if amounts.is_empty() {
        return None;
    }
    let min = *amounts.iter().min().unwrap();
    let max = *amounts.iter().max().unwrap();
    let currency = if text.contains('$') {
        "USD"
    } else if text.contains('£') {
        "GBP"
    } else {
        "EUR"
    };
    Some((min, max, currency.to_string()))
}

/// Extrait des mots-clés saillants (fréquence) depuis un texte, hors stopwords.
pub fn extract_keywords(text: &str, max: usize) -> Vec<String> {
    use std::collections::HashMap;
    const STOPWORDS: &[&str] = &[
        "le", "la", "les", "un", "une", "des", "de", "du", "et", "en", "à", "a", "au", "aux",
        "pour", "par", "sur", "dans", "avec", "sans", "ou", "où", "que", "qui", "quoi", "dont",
        "est", "sont", "être", "avoir", "plus", "the", "and", "for", "with", "your", "you", "our",
        "this", "that", "from", "are", "will", "you'll", "nous", "vous", "ils", "elles", "notre",
        "votre", "leur", "poste", "offre", "emploi", "job", "cdi", "cdd", "ans",
    ];
    let re = Regex::new(r"[a-zà-ÿA-ZÀ-Ÿ][a-zà-ÿA-ZÀ-Ÿ\-]{2,}").unwrap();
    let lower = normalize(text);
    let mut freq: HashMap<String, usize> = HashMap::new();
    for m in re.find_iter(&lower) {
        let w = m.as_str().to_string();
        if w.len() < 3 || STOPWORDS.contains(&w.as_str()) {
            continue;
        }
        *freq.entry(w).or_insert(0) += 1;
    }
    let mut items: Vec<(String, usize)> = freq.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    items.into_iter().take(max).map(|(w, _)| w).collect()
}

/// Construit un résumé court à partir d'un texte long.
pub fn summarize(text: &str, max_chars: usize) -> String {
    let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.chars().count() <= max_chars {
        return clean;
    }
    let truncated: String = clean.chars().take(max_chars).collect();
    match truncated.rfind(' ') {
        Some(idx) => format!("{}…", &truncated[..idx]),
        None => format!("{truncated}…"),
    }
}

/// Divise un texte en sections exploitables (pour l'analyse CV).
pub fn find_emails(text: &str) -> Vec<String> {
    let re = Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").unwrap();
    re.find_iter(text).map(|m| m.as_str().to_string()).collect()
}

pub fn find_phones(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?:\+33|0)\s?[1-9](?:[\s.\-]?\d{2}){4}").unwrap();
    re.find_iter(text).map(|m| m.as_str().trim().to_string()).collect()
}
