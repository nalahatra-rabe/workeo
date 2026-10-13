use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{Application, CreateApplication, Job, UpdateApplication};
use crate::state::AppState;

#[derive(Serialize)]
pub struct ApplicationWithJob {
    #[serde(flatten)]
    pub application: Application,
    pub job: Option<Job>,
}

/// Vérifie que le profil appartient à l'utilisateur.
async fn owned_profile(state: &AppState, user_id: &str, profile_id: &str) -> AppResult<()> {
    let p = db::get_profile(&state.pool, profile_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if p.user_id != user_id {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// POST /api/applications — enregistre ou met à jour une candidature.
pub async fn create_application(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateApplication>,
) -> AppResult<Json<Application>> {
    if input.profile_id.trim().is_empty() {
        return Err(AppError::BadRequest("profile_id requis".into()));
    }
    owned_profile(&state, &auth.user_id, &input.profile_id).await?;

    // L'offre doit exister.
    db::get_job(&state.pool, &input.job_id).await?.ok_or(AppError::NotFound)?;

    let app = db::upsert_application(
        &state.pool,
        &input.profile_id,
        &input.job_id,
        &input.status.unwrap_or_else(|| "saved".into()),
        input.notes.as_deref(),
    )
    .await?;
    Ok(Json(app))
}

/// PATCH /api/applications/:id — met à jour statut / notes.
pub async fn update_application(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateApplication>,
) -> AppResult<Json<Application>> {
    let row = sqlx::query("SELECT * FROM applications WHERE id=?1")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let existing = db::row_to_application(&row);
    owned_profile(&state, &auth.user_id, &existing.profile_id).await?;

    let app = db::upsert_application(
        &state.pool,
        &existing.profile_id,
        &existing.job_id,
        &input.status.unwrap_or(existing.status),
        input.notes.as_deref().or(existing.notes.as_deref()),
    )
    .await?;
    Ok(Json(app))
}

/// GET /api/profiles/:id/applications — suivi enrichi des offres.
pub async fn list_applications(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<ApplicationWithJob>>> {
    owned_profile(&state, &auth.user_id, &id).await?;

    let rows = sqlx::query(
        "SELECT * FROM applications WHERE profile_id=?1 ORDER BY updated_at DESC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await?;

    let mut out = Vec::new();
    for row in &rows {
        let application = db::row_to_application(row);
        let job = db::get_job(&state.pool, &application.job_id).await?;
        out.push(ApplicationWithJob { application, job });
    }
    Ok(Json(out))
}

/// DELETE /api/applications/:id — supprime une candidature.
pub async fn delete_application(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let row = sqlx::query("SELECT * FROM applications WHERE id=?1")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let existing = db::row_to_application(&row);
    owned_profile(&state, &auth.user_id, &existing.profile_id).await?;

    sqlx::query("DELETE FROM applications WHERE id=?1")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
