use axum::extract::{Multipart, Path, State};
use axum::Json;
use chrono::Utc;
use sqlx::Row;

use crate::agents::orchestrator::SearchOrchestrator;
use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{CreateProfile, Profile, ScoredJob, UpdateProfile};
use crate::services::cv_parser;
use crate::state::AppState;

/// Nombre maximal de profils par utilisateur.
const MAX_PROFILES: i64 = 10;

/// Charge un profil en vérifiant qu'il appartient bien à l'utilisateur.
async fn owned_profile(state: &AppState, user_id: &str, id: &str) -> AppResult<Profile> {
    let p = db::get_profile(&state.pool, id).await?.ok_or(AppError::NotFound)?;
    if p.user_id != user_id {
        return Err(AppError::NotFound);
    }
    Ok(p)
}

/// POST /api/profiles — crée un nouveau profil pour l'utilisateur connecté.
pub async fn create_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateProfile>,
) -> AppResult<Json<Profile>> {
    if input.full_name.trim().is_empty() {
        return Err(AppError::BadRequest("le nom complet est requis".into()));
    }

    let count = db::count_profiles(&state.pool, &auth.user_id).await?;
    if count >= MAX_PROFILES {
        return Err(AppError::BadRequest(format!(
            "limite de {MAX_PROFILES} profils atteinte"
        )));
    }

    let now = Utc::now();
    let is_default = input.is_default.unwrap_or(count == 0);
    let name = input
        .name
        .filter(|n| !n.trim().is_empty())
        .or_else(|| input.headline.clone())
        .unwrap_or_else(|| format!("Profil {}", count + 1));

    let mut profile = Profile {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: auth.user_id.clone(),
        name,
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
        is_default,
        created_at: now,
        updated_at: now,
    };

    db::insert_profile(&state.pool, &profile).await?;
    if is_default {
        db::set_default_profile(&state.pool, &auth.user_id, &profile.id).await?;
    }

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

/// GET /api/profiles — liste tous les profils de l'utilisateur.
pub async fn list_profiles(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<Profile>>> {
    let profiles = db::list_profiles(&state.pool, &auth.user_id).await?;
    Ok(Json(profiles))
}

/// GET /api/profiles/:id
pub async fn get_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Profile>> {
    Ok(Json(owned_profile(&state, &auth.user_id, &id).await?))
}

/// PUT /api/profiles/:id — mise à jour partielle.
pub async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateProfile>,
) -> AppResult<Json<Profile>> {
    let mut profile = owned_profile(&state, &auth.user_id, &id).await?;

    if let Some(v) = input.name {
        if !v.trim().is_empty() {
            profile.name = v;
        }
    }
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

    if input.is_default == Some(true) {
        db::set_default_profile(&state.pool, &auth.user_id, &profile.id).await?;
        profile.is_default = true;
    }

    Ok(Json(profile))
}

/// POST /api/profiles/:id/default — définit ce profil comme profil actif.
pub async fn set_default(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Profile>> {
    let profile = owned_profile(&state, &auth.user_id, &id).await?;
    db::set_default_profile(&state.pool, &auth.user_id, &profile.id).await?;
    let updated = db::get_profile(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(updated))
}

/// DELETE /api/profiles/:id — supprime un profil et ses données liées.
pub async fn delete_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let was_default = owned_profile(&state, &auth.user_id, &id).await?.is_default;
    let ok = db::delete_profile(&state.pool, &auth.user_id, &id).await?;
    if !ok {
        return Err(AppError::NotFound);
    }
    // Si on a supprimé le profil par défaut, on promeut le premier restant.
    if was_default {
        let remaining = db::list_profiles(&state.pool, &auth.user_id).await?;
        if let Some(first) = remaining.first() {
            db::set_default_profile(&state.pool, &auth.user_id, &first.id).await?;
        }
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// POST /api/profiles/:id/cv — upload CV (multipart) + analyse IA.
pub async fn upload_cv(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> AppResult<Json<Profile>> {
    let mut profile = owned_profile(&state, &auth.user_id, &id).await?;

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
        let fname = filename.clone();
        tokio::task::spawn_blocking(move || cv_parser::extract_text(&fname, &b))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("tâche bloquée: {e}")))?
            .map_err(|e| AppError::BadRequest(format!("extraction CV impossible: {e}")))?
    } else if let Some(t) = inline_text {
        filename = "cv.txt".into();
        t
    } else {
        return Err(AppError::BadRequest("aucun fichier CV ou texte fourni".into()));
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

    SearchOrchestrator::analyze_profile(&state, &mut profile)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(profile))
}

/// POST /api/profiles/:id/analyze — (ré)analyse le profil avec l'IA.
pub async fn analyze_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Profile>> {
    let mut profile = owned_profile(&state, &auth.user_id, &id).await?;
    SearchOrchestrator::analyze_profile(&state, &mut profile)
        .await
        .map_err(AppError::Internal)?;
    let updated = db::get_profile(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(updated))
}

/// GET /api/profiles/:id/recommendations — meilleures offres pour ce profil.
pub async fn recommendations(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<ScoredJob>>> {
    owned_profile(&state, &auth.user_id, &id).await?;

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
            let reasons: Vec<String> = serde_json::from_str(
                row.get::<Option<String>, _>("match_reasons").as_deref().unwrap_or("[]"),
            )
            .unwrap_or_default();
            ScoredJob { job, score, reasons }
        })
        .collect();
    out.sort_by(|a, b| b.score.cmp(&a.score));

    Ok(Json(out))
}
