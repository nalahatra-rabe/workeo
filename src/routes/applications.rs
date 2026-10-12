use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;

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

/// POST /api/applications — enregistre ou met à jour une candidature.
pub async fn create_application(
    State(state): State<AppState>,
    Json(input): Json<CreateApplication>,
) -> AppResult<Json<Application>> {
    if input.profile_id.trim().is_empty() || input.job_id.trim().is_empty() {
        return Err(AppError::BadRequest("profile_id et job_id requis".into()));
    }
    // Vérifie que l'offre existe.
    if db::get_job(&state.pool, &input.job_id).await?.is_none() {
        return Err(AppError::NotFound);
    }
    let status = input.status.unwrap_or_else(|| "saved".into());
    let app = db::upsert_application(
        &state.pool,
        &input.profile_id,
        &input.job_id,
        &status,
        input.notes.as_deref(),
    )
    .await?;
    Ok(Json(app))
}

/// PATCH /api/applications/:id — met à jour statut / notes.
pub async fn update_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateApplication>,
) -> AppResult<Json<Application>> {
    let row = sqlx::query("SELECT * FROM applications WHERE id=?1")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let existing = db::row_to_application(&row);

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
    Path(id): Path<String>,
) -> AppResult<Json<Vec<ApplicationWithJob>>> {
    let rows = sqlx::query(
        "SELECT * FROM applications WHERE profile_id=?1 ORDER BY updated_at DESC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await?;

    let mut out = Vec::new();
    for row in rows {
        let application = db::row_to_application(&row);
        let job = db::get_job(&state.pool, &application.job_id).await?;
        out.push(ApplicationWithJob { application, job });
    }
    Ok(Json(out))
}

/// DELETE /api/applications/:id
pub async fn delete_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let res = sqlx::query("DELETE FROM applications WHERE id=?1")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
