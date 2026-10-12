use crate::agents::heuristics;
use crate::agents::llm::LlmClient;
use crate::models::{Profile, ProfileCategorization};

/// Agent 2 — Catégorisation du profil.
///
/// Détermine la catégorie métier principale, les catégories secondaires, les
/// mots-clés de recherche et les titres de poste à viser.
pub struct ProfileCategorizerAgent<'a> {
    llm: &'a LlmClient,
}

impl<'a> ProfileCategorizerAgent<'a> {
    pub fn new(llm: &'a LlmClient) -> Self {
        ProfileCategorizerAgent { llm }
    }

    pub async fn categorize(&self, profile: &Profile) -> ProfileCategorization {
        if self.llm.is_enabled() {
            if let Some(cat) = self.categorize_with_llm(profile).await {
                return self.reconcile(cat, profile);
            }
        }
        self.categorize_heuristically(profile)
    }

    async fn categorize_with_llm(&self, profile: &Profile) -> Option<ProfileCategorization> {
        let system = "Tu es un expert en orientation professionnelle. À partir du profil, \
            catégorise-le et réponds en JSON STRICT: \
            {\"primary_category\":string,\"categories\":[string],\
            \"search_keywords\":[string],\"suggested_titles\":[string],\
            \"remote_preference\":\"on-site|hybrid|remote|any\"}. \
            search_keywords = 5 à 10 mots-clés efficaces pour trouver des offres sur un job board (en français et anglais si pertinent). \
            suggested_titles = intitulés de poste réalistes et immédiatement candidatables (6 à 10).";

        let ctx = profile_context(profile);
        let user = format!("Profil candidat:\n{ctx}");

        let v = self.llm.complete_json(system, &user).await?;
        serde_json::from_value(v).ok()
    }

    fn categorize_heuristically(&self, profile: &Profile) -> ProfileCategorization {
        // Texte combiné pour la détection.
        let mut blob = String::new();
        blob.push_str(profile.headline.as_deref().unwrap_or(""));
        blob.push(' ');
        blob.push_str(profile.summary.as_deref().unwrap_or(""));
        blob.push(' ');
        blob.push_str(&profile.skills.join(" "));
        blob.push(' ');
        for exp in &profile.experiences {
            blob.push_str(exp.title.as_deref().unwrap_or(""));
            blob.push(' ');
            blob.push_str(exp.description.as_deref().unwrap_or(""));
            blob.push(' ');
        }
        blob.push_str(&profile.domains.join(" "));
        blob.push_str(profile.cv_text.as_deref().unwrap_or(""));

        let primary = heuristics::detect_category(&blob);

        // Catégories secondaires : toutes celles détectées.
        let mut categories: Vec<String> = Vec::new();
        let lower_blob = heuristics::normalize(&blob);
        for (cat, kws) in heuristics::CATEGORIES.iter() {
            if kws
                .iter()
                .any(|k| heuristics::contains_bounded(&lower_blob, &k.to_lowercase()))
            {
                categories.push((*cat).to_string());
            }
        }
        if let Some(p) = &primary {
            categories.retain(|c| c != p);
            categories.insert(0, p.clone());
        }
        categories.truncate(4);

        // Titres suggérés : à partir de la séniorité + catégorie + skills.
        let seniority = profile.seniority.clone().unwrap_or_else(|| "Confirmé".into());
        let mut titles: Vec<String> = Vec::new();
        titles.extend(profile.target_roles.clone());
        if let Some(headline) = &profile.headline {
            if headline.len() < 60 {
                titles.push(headline.clone());
            }
        }
        for base in default_titles_for(primary.as_deref()) {
            titles.push(base.to_string());
        }

        // Mots-clés : titre principal + top skills + catégorie.
        let mut keywords: Vec<String> = Vec::new();
        if let Some(t) = titles.first() {
            keywords.push(t.clone());
        }
        keywords.extend(profile.skills.iter().take(4).cloned());
        if let Some(c) = &primary {
            keywords.push(c.split('/').last().unwrap_or(c).trim().to_string());
        }
        keywords = dedup(keywords);
        keywords.truncate(8);

        let remote_preference = profile.remote_preference.clone().or(Some("any".into()));
        let _ = seniority;

        ProfileCategorization {
            primary_category: primary.clone(),
            categories,
            search_keywords: keywords,
            suggested_titles: dedup(titles),
            remote_preference,
        }
    }

    fn reconcile(&self, mut c: ProfileCategorization, profile: &Profile) -> ProfileCategorization {
        if c.primary_category.is_none() {
            c.primary_category = heuristics::detect_category(
                &format!("{} {}", profile.headline.clone().unwrap_or_default(), profile.skills.join(" ")),
            );
        }
        if c.search_keywords.is_empty() {
            c.search_keywords.extend(profile.skills.iter().take(5).cloned());
        }
        if c.suggested_titles.is_empty() {
            c.suggested_titles = profile.target_roles.clone();
        }
        c.search_keywords = dedup(c.search_keywords);
        c.suggested_titles = dedup(c.suggested_titles);
        c
    }
}

fn profile_context(p: &Profile) -> String {
    let mut s = String::new();
    s.push_str(&format!("Nom: {}\n", p.full_name));
    if let Some(h) = &p.headline { s.push_str(&format!("Titre: {h}\n")); }
    if let Some(sm) = &p.summary { s.push_str(&format!("Résumé: {sm}\n")); }
    if let Some(se) = &p.seniority { s.push_str(&format!("Séniorité: {se}\n")); }
    if let Some(y) = p.years_experience { s.push_str(&format!("Années d'expérience: {y}\n")); }
    if !p.skills.is_empty() { s.push_str(&format!("Compétences: {}\n", p.skills.join(", "))); }
    if !p.languages.is_empty() {
        let l: Vec<String> = p.languages.iter().map(|x| x.name.clone()).collect();
        s.push_str(&format!("Langues: {}\n", l.join(", ")));
    }
    if !p.education.is_empty() {
        let e: Vec<String> = p.education.iter().filter_map(|x| x.degree.clone()).collect();
        s.push_str(&format!("Formation: {}\n", e.join(", ")));
    }
    if !p.experiences.is_empty() {
        s.push_str("Expériences:\n");
        for e in p.experiences.iter().take(6) {
            s.push_str(&format!(" - {} @ {}\n", e.title.clone().unwrap_or_default(), e.company.clone().unwrap_or_default()));
        }
    }
    if !p.target_roles.is_empty() { s.push_str(&format!("Postes visés: {}\n", p.target_roles.join(", "))); }
    if let Some(r) = &p.remote_preference { s.push_str(&format!("Télétravail: {r}\n")); }
    if !p.contract_types.is_empty() { s.push_str(&format!("Contrats recherchés: {}\n", p.contract_types.join(", "))); }
    s
}

fn default_titles_for(category: Option<&str>) -> &'static [&'static str] {
    match category {
        Some("Informatique / Développement") => &["Développeur Fullstack", "Développeur Web", "Ingénieur logiciel"],
        Some("Data / IA") => &["Data Analyst", "Data Scientist", "Ingénieur Data"],
        Some("Design / UX") => &["UX/UI Designer", "Product Designer", "Graphiste"],
        Some("Produit / Gestion de projet") => &["Chef de projet", "Product Owner", "Scrum Master"],
        Some("Marketing / Communication") => &["Chargé de communication", "Community Manager", "Growth Marketer"],
        Some("Commerce / Vente") => &["Commercial", "Conseiller de vente", "Business Developer"],
        Some("Finance / Comptabilité") => &["Comptable", "Assistant comptable", "Contrôleur de gestion"],
        Some("Ressources humaines") => &["Chargé de recrutement", "Assistant RH"],
        Some("Santé / Social") => &["Aide-soignant", "Éducateur spécialisé", "Assistant social"],
        Some("Éducation / Formation") => &["Formateur", "Enseignant", "Animateur"],
        Some("Hôtellerie / Restauration / Tourisme") => &["Serveur", "Cuisinier", "Réceptionniste"],
        Some("Logistique / Transport") => &["Chauffeur", "Préparateur de commandes", "Magasinier"],
        Some("Industrie / Bâtiment / Technique") => &["Technicien de maintenance", "Opérateur", "Électricien"],
        Some("Sécurité / Accueil") => &["Agent de sécurité", "Agent d'accueil"],
        Some("Juridique / Administratif") => &["Assistant administratif", "Secrétaire", "Assistant juridique"],
        _ => &["Chargé de mission", "Assistant", "Coordinateur"],
    }
}

fn dedup(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    items
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.to_lowercase()))
        .collect()
}
