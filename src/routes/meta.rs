use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::scraper::known_sources;
use crate::state::AppState;

/// GET /api/health — état du service et de l'IA.
pub async fn health(State(state): State<AppState>) -> Json<Value> {
    let llm_enabled = state.llm.is_enabled();
    let reachable = if llm_enabled { state.llm.health().await } else { false };

    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "llm": {
            "provider": state.llm.provider_label(),
            "model": state.llm.model_label(),
            "enabled": llm_enabled,
            "reachable": reachable,
            "mode": if reachable { "llm" } else { "heuristique" },
        },
        "sources": known_sources().iter().map(|(n, l)| json!({"name": n, "label": l})).collect::<Vec<_>>(),
    }))
}

/// GET /api/sources — liste des sources et leur activation.
pub async fn sources(State(state): State<AppState>) -> Json<Value> {
    let active = &state.config.scrape_sources;
    Json(json!({
        "active_default": active,
        "sources": known_sources().iter().map(|(n, l)| json!({
            "name": n,
            "label": l,
            "enabled": active.iter().any(|a| a == n),
        })).collect::<Vec<_>>(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub profile_id: Option<String>,
}

/// GET /api/stats — statistiques pour le tableau de bord.
/// Scopées au profil fourni (`?profile_id=`) lorsque présent.
pub async fn stats(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<StatsQuery>,
) -> AppResult<Json<Value>> {
    let jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs")
        .fetch_one(&state.pool)
        .await?;
    let profiles: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM profiles WHERE user_id = ?1")
        .bind(&auth.user_id)
        .fetch_one(&state.pool)
        .await?;

    let mut matches = 0i64;
    let mut applications = 0i64;
    let mut searches = 0i64;
    let mut by_status: Vec<Value> = Vec::new();

    if let Some(pid) = q.profile_id.as_deref() {
        if let Some(p) = crate::db::get_profile(&state.pool, pid).await? {
            if p.user_id != auth.user_id {
                return Err(AppError::NotFound);
            }
            matches = sqlx::query_scalar("SELECT COUNT(*) FROM matches WHERE profile_id = ?1")
                .bind(pid)
                .fetch_one(&state.pool)
                .await?;
            applications =
                sqlx::query_scalar("SELECT COUNT(*) FROM applications WHERE profile_id = ?1")
                    .bind(pid)
                    .fetch_one(&state.pool)
                    .await?;
            searches = sqlx::query_scalar("SELECT COUNT(*) FROM search_runs WHERE profile_id = ?1")
                .bind(pid)
                .fetch_one(&state.pool)
                .await?;

            let rows = sqlx::query(
                "SELECT status, COUNT(*) AS n FROM applications WHERE profile_id = ?1 GROUP BY status",
            )
            .bind(pid)
            .fetch_all(&state.pool)
            .await?;
            by_status = rows
                .iter()
                .map(|r| {
                    json!({
                        "status": r.get::<String, _>("status"),
                        "count": r.get::<i64, _>("n"),
                    })
                })
                .collect();
        } else {
            return Err(AppError::NotFound);
        }
    }

    let remote: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE remote = 1")
        .fetch_one(&state.pool)
        .await?;

    let rows = sqlx::query(
        "SELECT category, COUNT(*) AS n FROM jobs WHERE category IS NOT NULL GROUP BY category ORDER BY n DESC LIMIT 12",
    )
    .fetch_all(&state.pool)
    .await?;

    let by_category: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "category": r.get::<Option<String>, _>("category"),
                "count": r.get::<i64, _>("n"),
            })
        })
        .collect();

    Ok(Json(json!({
        "jobs": jobs,
        "profiles": profiles,
        "remote_jobs": remote,
        "matches": matches,
        "applications": applications,
        "searches": searches,
        "by_status": by_status,
        "by_category": by_category,
    })))
}

/// GET /api/categories — catégories métier connues (référentiel).
pub async fn categories(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let mut cats: Vec<String> = crate::agents::heuristics::CATEGORIES
        .iter()
        .map(|(c, _)| c.to_string())
        .collect();
    // Ajoute les catégories présentes en base mais absentes du référentiel.
    let rows = sqlx::query("SELECT DISTINCT category FROM jobs WHERE category IS NOT NULL")
        .fetch_all(&state.pool)
        .await?;
    for r in rows {
        if let Some(c) = r.get::<Option<String>, _>("category") {
            if !cats.contains(&c) {
                cats.push(c);
            }
        }
    }
    Ok(Json(json!(cats)))
}
