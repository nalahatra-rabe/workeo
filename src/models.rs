use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────
// Utilisateurs & authentification
// ─────────────────────────────────────────────────────────────

/// Compte utilisateur (propriétaire d'un ou plusieurs profils).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub full_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

/// Claims du JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Identifiant utilisateur.
    pub sub: String,
    /// Expiration (timestamp UNIX).
    pub exp: usize,
}

// ─────────────────────────────────────────────────────────────
// Profil utilisateur
// ─────────────────────────────────────────────────────────────

/// Représente un profil candidat tel que stocké en base.
/// Un utilisateur peut posséder plusieurs profils (données distinctes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub user_id: String,
    /// Nom du profil (ex: "Développeur", "Alternance marketing").
    pub name: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub headline: Option<String>,
    pub summary: Option<String>,
    pub seniority: Option<String>,
    pub years_experience: Option<f64>,
    /// Compétences techniques & humaines.
    pub skills: Vec<String>,
    pub languages: Vec<Language>,
    pub education: Vec<Education>,
    pub experiences: Vec<Experience>,
    /// Domaines métiers détectés (ex: "Développement web", "Comptabilité").
    pub domains: Vec<String>,
    /// Titres de poste ciblés suggérés par l'IA.
    pub target_roles: Vec<String>,
    /// Mots-clés de recherche générés.
    pub search_keywords: Vec<String>,
    /// Catégorie principale (ex: "Informatique / Dev").
    pub category: Option<String>,
    /// Préférences
    pub remote_preference: Option<String>, // on-site | hybrid | remote | any
    pub contract_types: Vec<String>,       // CDI, CDD, stage, alternance, freelance
    pub salary_min: Option<i64>,
    /// CV source
    pub cv_filename: Option<String>,
    pub cv_text: Option<String>,
    /// Profil actif par défaut pour l'utilisateur.
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Language {
    pub name: String,
    pub level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Education {
    pub degree: Option<String>,
    pub school: Option<String>,
    pub year: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Experience {
    pub title: Option<String>,
    pub company: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub description: Option<String>,
}

/// Payload de création de profil (infos minimales ; le reste vient du CV/IA).
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProfile {
    pub name: Option<String>,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub headline: Option<String>,
    pub remote_preference: Option<String>,
    #[serde(default)]
    pub contract_types: Vec<String>,
    pub salary_min: Option<i64>,
    /// Le cas échéant, texte de CV fourni directement.
    pub cv_text: Option<String>,
    pub is_default: Option<bool>,
}

/// Payload de mise à jour (tous champs optionnels).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateProfile {
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub headline: Option<String>,
    pub summary: Option<String>,
    pub remote_preference: Option<String>,
    pub contract_types: Option<Vec<String>>,
    pub salary_min: Option<i64>,
    pub skills: Option<Vec<String>>,
    pub target_roles: Option<Vec<String>>,
    pub is_default: Option<bool>,
}

// ─────────────────────────────────────────────────────────────
// Offres d'emploi
// ─────────────────────────────────────────────────────────────

/// Offre brute issue du scraping, avant catégorisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawJob {
    pub source: String,
    pub external_id: String,
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub remote: Option<bool>,
    pub salary: Option<String>,
    pub contract_type: Option<String>,
    pub tags: Vec<String>,
    pub posted_at: Option<DateTime<Utc>>,
}

/// Offre normalisée et catégorisée, prête pour le matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub source: String,
    pub external_id: String,
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub remote: bool,
    pub description: Option<String>,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub salary_currency: Option<String>,
    pub contract_type: Option<String>,
    pub seniority: Option<String>,
    pub category: Option<String>,
    pub skills: Vec<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub scraped_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────
// Matchs & candidatures
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub profile_id: String,
    pub job_id: String,
    pub score: i64,
    pub reasons: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Offre enrichie de son score de compatibilité (réponse API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredJob {
    #[serde(flatten)]
    pub job: Job,
    pub score: i64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: String,
    pub profile_id: String,
    pub job_id: String,
    pub status: String, // saved | applied | interview | offer | rejected
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApplication {
    pub profile_id: String,
    pub job_id: String,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateApplication {
    pub status: Option<String>,
    pub notes: Option<String>,
}

// ─────────────────────────────────────────────────────────────
// Recherche
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    pub profile_id: String,
    /// Requête libre ; si absente, dérivée du profil par l'orchestrateur.
    pub query: Option<String>,
    pub location: Option<String>,
    pub remote_only: Option<bool>,
    pub contract_type: Option<String>,
    /// Latitude/longitude issues de la géolocalisation navigateur (optionnel).
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    /// Sources à interroger ; si vide, la config par défaut est utilisée.
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRun {
    pub id: String,
    pub profile_id: String,
    pub query: Option<String>,
    pub location: Option<String>,
    pub sources: Vec<String>,
    pub status: String,
    pub jobs_found: i64,
    pub jobs_new: i64,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub run: SearchRun,
    pub total: i64,
    pub results: Vec<ScoredJob>,
    pub source_errors: Vec<String>,
}

// ─────────────────────────────────────────────────────────────
// Sorties structurées des agents IA
// ─────────────────────────────────────────────────────────────

/// Résultat de l'analyse du CV par le `CvAnalyzerAgent`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CvAnalysis {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub headline: Option<String>,
    pub summary: Option<String>,
    pub seniority: Option<String>,
    pub years_experience: Option<f64>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub languages: Vec<Language>,
    #[serde(default)]
    pub education: Vec<Education>,
    #[serde(default)]
    pub experiences: Vec<Experience>,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub strengths: Vec<String>,
}

/// Résultat du `ProfileCategorizerAgent`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileCategorization {
    pub primary_category: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub search_keywords: Vec<String>,
    #[serde(default)]
    pub suggested_titles: Vec<String>,
    pub remote_preference: Option<String>,
}

/// Retour du `JobCategorizerAgent` pour une offre.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobCategorization {
    pub external_id: String,
    pub category: Option<String>,
    pub seniority: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    pub contract_type: Option<String>,
    pub remote: Option<bool>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub salary_currency: Option<String>,
    pub summary: Option<String>,
}

/// Retour du `MatcherAgent` pour une offre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchScore {
    pub job_id: String,
    pub score: i64,
    #[serde(default)]
    pub reasons: Vec<String>,
}
