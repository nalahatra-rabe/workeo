use axum::extract::State;
use axum::Json;

use crate::agents::orchestrator::SearchOrchestrator;
use crate::error::{AppError, AppResult};
use crate::models::{SearchRequest, SearchResult};
use crate::state::AppState;

/// POST /api/search — lance une recherche multi-agents complète.
pub async fn search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> AppResult<Json<SearchResult>> {
    if req.profile_id.trim().is_empty() {
        return Err(AppError::BadRequest("profile_id requis".into()));
    }
    let result = SearchOrchestrator::run(&state, &req)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(result))
}

/// GET /api/profiles/:id/searches — historique des recherches.
pub async fn history(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> AppResult<Json<Vec<crate::models::SearchRun>>> {
    let rows = sqlx::query("SELECT * FROM search_runs WHERE profile_id=?1 ORDER BY started_at DESC LIMIT 20")
        .bind(&id)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows.iter().map(crate::db::row_to_search_run).collect()))
}
