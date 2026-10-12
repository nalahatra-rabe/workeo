use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{Job, ScoredJob};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    pub q: Option<String>,
    pub remote: Option<bool>,
    pub contract: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/jobs — liste filtrée d'offres (exploration libre).
pub async fn list_jobs(
    State(state): State<AppState>,
    Query(q): Query<ListJobsQuery>,
) -> AppResult<Json<Vec<Job>>> {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);

    let mut sql = String::from("SELECT * FROM jobs WHERE 1=1");
    let mut bind_text: Vec<String> = Vec::new();

    if let Some(text) = &q.q {
        if !text.trim().is_empty() {
            sql.push_str(" AND (LOWER(title) LIKE ?1 OR LOWER(company) LIKE ?1 OR LOWER(description) LIKE ?1)");
            bind_text.push(format!("%{}%", text.to_lowercase()));
        }
    }
    if q.remote == Some(true) {
        sql.push_str(" AND remote = 1");
    }
    if let Some(c) = &q.contract {
        sql.push_str(&format!(" AND LOWER(contract_type) LIKE '%{}%'", sanitize(c)));
    }
    if let Some(c) = &q.category {
        sql.push_str(&format!(" AND LOWER(category) LIKE '%{}%'", sanitize(c)));
    }
    sql.push_str(&format!(" ORDER BY scraped_at DESC LIMIT {limit} OFFSET {offset}"));

    let mut query = sqlx::query(&sql);
    for b in &bind_text {
        query = query.bind(b);
    }
    let rows = query.fetch_all(&state.pool).await?;
    Ok(Json(rows.iter().map(db::row_to_job).collect()))
}

/// GET /api/jobs/:id — détail d'une offre (enrichi du score si profil fourni).
pub async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<JobDetailQuery>,
) -> AppResult<Json<ScoredJob>> {
    let job = db::get_job(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    let (score, reasons) = if let Some(pid) = params.profile_id {
        match db::get_match(&state.pool, &pid, &id).await? {
            Some(m) => (m.score, m.reasons),
            None => (0, vec![]),
        }
    } else {
        (0, vec![])
    };
    Ok(Json(ScoredJob { job, score, reasons }))
}

#[derive(Debug, Deserialize)]
pub struct JobDetailQuery {
    pub profile_id: Option<String>,
}

fn sanitize(s: &str) -> String {
    s.to_lowercase().replace('\'', "").replace(';', "")
}
