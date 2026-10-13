use axum::extract::{Path, State};
use axum::Json;

use crate::agents::orchestrator::SearchOrchestrator;
use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{SearchRequest, SearchResult};
use crate::state::AppState;

/// POST /api/search — lance une recherche multi-agents complète.
pub async fn search(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<SearchRequest>,
) -> AppResult<Json<SearchResult>> {
    if req.profile_id.trim().is_empty() {
        return Err(AppError::BadRequest("profile_id requis".into()));
    }
    // Vérifie que le profil appartient à l'utilisateur.
    let profile = db::get_profile(&state.pool, &req.profile_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if profile.user_id != auth.user_id {
        return Err(AppError::NotFound);
    }

    let result = SearchOrchestrator::run(&state, &req)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(result))
}

/// GET /api/profiles/:id/searches — historique des recherches.
pub async fn history(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<crate::models::SearchRun>>> {
    let p = db::get_profile(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    if p.user_id != auth.user_id {
        return Err(AppError::NotFound);
    }
    let rows = sqlx::query(
        "SELECT * FROM search_runs WHERE profile_id=?1 ORDER BY started_at DESC LIMIT 20",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows.iter().map(db::row_to_search_run).collect()))
}
