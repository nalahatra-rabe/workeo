use axum::extract::{Multipart, Path, State};
use axum::Json;
use chrono::Utc;
use sqlx::Row;

use crate::agents::orchestrator::SearchOrchestrator;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{CreateProfile, Profile, ScoredJob, UpdateProfile};
use crate::services::cv_parser;
use crate::state::AppState;

/// POST /api/profiles — crée un profil.
pub async fn create_profile(
    State(state): State<AppState>,
    Json(input): Json<CreateProfile>,
) -> AppResult<Json<Profile>> {
    if input.full_name.trim().is_empty() {
        return Err(AppError::BadRequest("le nom complet est requis".into()));
    }
    let now = Utc::now();
    let mut profile = Profile {
        id: uuid::Uuid::new_v4().to_string(),
        full_name: input.full_name.trim().to_string(),
        email: input.email,
        phone: input.phone,
        location: input.location,
        headline: input.headline,
        summary: None,
        seniority: None,
        years_experience: None,
        skills: Vec::new(),
        languages: Vec::new(),
        education: Vec::new(),
        experiences: Vec::new(),
        domains: Vec::new(),
        target_roles: Vec::new(),
        search_keywords: Vec::new(),
        category: None,
        remote_preference: input.remote_preference,
        contract_types: input.contract_types,
        salary_min: input.salary_min,
        cv_filename: None,
        cv_text: input.cv_text,
        created_at: now,
        updated_at: now,
    };

    db::insert_profile(&state.pool, &profile).await?;

    // Si un CV texte est fourni dès la création, on l'analyse.
    if profile
        .cv_text
        .as_deref()
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false)
    {
        if let Err(e) = SearchOrchestrator::analyze_profile(&state, &mut profile).await {
            tracing::warn!("analyse à la création échouée: {e}");
        }
    }

    Ok(Json(profile))
}

/// GET /api/profiles/:id
pub async fn get_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Profile>> {
    db::get_profile(&state.pool, &id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

/// PUT /api/profiles/:id — mise à jour partielle.
pub async fn update_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateProfile>,
) -> AppResult<Json<Profile>> {
    let mut profile = db::get_profile(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    if let Some(v) = input.full_name {
        profile.full_name = v;
    }
    if let Some(v) = input.email {
        profile.email = Some(v);
    }
    if let Some(v) = input.phone {
        profile.phone = Some(v);
    }
    if let Some(v) = input.location {
        profile.location = Some(v);
    }
    if let Some(v) = input.headline {
        profile.headline = Some(v);
    }
    if let Some(v) = input.summary {
        profile.summary = Some(v);
    }
    if let Some(v) = input.remote_preference {
        profile.remote_preference = Some(v);
    }
    if let Some(v) = input.contract_types {
        profile.contract_types = v;
    }
    if let Some(v) = input.salary_min {
        profile.salary_min = Some(v);
    }
    if let Some(v) = input.skills {
        profile.skills = v;
    }
    if let Some(v) = input.target_roles {
        profile.target_roles = v;
    }
    profile.updated_at = Utc::now();
    db::update_profile(&state.pool, &profile).await?;
    Ok(Json(profile))
}

/// POST /api/profiles/:id/cv — upload d'un CV (multipart) + analyse par l'IA.
pub async fn upload_cv(
    State(state): State<AppState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> AppResult<Json<Profile>> {
    // Charge d'abord le profil pour valider l'existence.
    let mut profile = db::get_profile(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut filename = String::from("cv");
    let mut bytes: Option<Vec<u8>> = None;
    let mut inline_text: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart invalide: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" | "cv" => {
                filename = field
                    .file_name()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "cv".into());
                bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("lecture fichier: {e}")))?
                        .to_vec(),
                );
            }
            "text" => {
                inline_text = field.text().await.ok();
            }
            _ => {}
        }
    }

    let text = if let Some(b) = bytes {
        // Parsing PDF/DOCX = CPU-bound → hors de l'event loop.
        let fname = filename.clone();
        tokio::task::spawn_blocking(move || cv_parser::extract_text(&fname, &b))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("tâche bloquée: {e}")))?
            .map_err(|e| AppError::BadRequest(format!("extraction CV impossible: {e}")))?
    } else if let Some(t) = inline_text {
        filename = "cv.txt".into();
        t
    } else {
        return Err(AppError::BadRequest(
            "aucun fichier CV ou texte fourni".into(),
        ));
    };

    if text.trim().len() < 30 {
        return Err(AppError::BadRequest(
            "le CV semble vide ou illisible (texte extrait trop court)".into(),
        ));
    }

    profile.cv_filename = Some(filename);
    profile.cv_text = Some(text);
    profile.updated_at = Utc::now();
    db::update_profile(&state.pool, &profile).await?;

    // Analyse IA : extraction + catégorisation.
    SearchOrchestrator::analyze_profile(&state, &mut profile)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(profile))
}

/// POST /api/profiles/:id/analyze — (ré)analyse le profil avec l'IA.
pub async fn analyze_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Profile>> {
    let mut profile = db::get_profile(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    SearchOrchestrator::analyze_profile(&state, &mut profile)
        .await
        .map_err(AppError::Internal)?;
    let updated = db::get_profile(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(updated))
}

/// GET /api/profiles/:id/recommendations — meilleures offres pour ce profil.
pub async fn recommendations(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<ScoredJob>>> {
    let limit: i64 = 40;
    let rows = sqlx::query(
        r#"SELECT j.*, m.score AS match_score, m.reasons AS match_reasons
           FROM matches m
           JOIN jobs j ON j.id = m.job_id
           WHERE m.profile_id = ?1
           ORDER BY m.score DESC, j.scraped_at DESC
           LIMIT ?2"#,
    )
    .bind(&id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;

    let mut out: Vec<ScoredJob> = rows
        .iter()
        .map(|row| {
            let job = db::row_to_job(row);
            let score: i64 = row.get("match_score");
            let reasons: Vec<String> =
                serde_json::from_str(row.get::<Option<String>, _>("match_reasons").as_deref().unwrap_or("[]"))
                    .unwrap_or_default();
            ScoredJob { job, score, reasons }
        })
        .collect();
    out.sort_by(|a, b| b.score.cmp(&a.score));

    Ok(Json(out))
}
