pub mod applications;
pub mod auth;
pub mod jobs;
pub mod meta;
pub mod profiles;
pub mod search;

use axum::routing::{delete, get, patch, post};
use axum::Router;

use crate::state::AppState;

/// Construit le routeur de l'API REST.
pub fn api_router() -> Router<AppState> {
    Router::new()
        // ─── Auth ───
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/ping", get(auth::protected_ping))
        // ─── Méta ───
        .route("/api/health", get(meta::health))
        .route("/api/sources", get(meta::sources))
        .route("/api/stats", get(meta::stats))
        .route("/api/categories", get(meta::categories))
        // ─── Profils (multi-profils) ───
        .route(
            "/api/profiles",
            get(profiles::list_profiles).post(profiles::create_profile),
        )
        .route(
            "/api/profiles/:id",
            get(profiles::get_profile)
                .put(profiles::update_profile)
                .delete(profiles::delete_profile),
        )
        .route("/api/profiles/:id/default", post(profiles::set_default))
        .route("/api/profiles/:id/cv", post(profiles::upload_cv))
        .route("/api/profiles/:id/analyze", post(profiles::analyze_profile))
        .route(
            "/api/profiles/:id/recommendations",
            get(profiles::recommendations),
        )
        .route("/api/profiles/:id/searches", get(search::history))
        .route(
            "/api/profiles/:id/applications",
            get(applications::list_applications),
        )
        // ─── Recherche ───
        .route("/api/search", post(search::search))
        // ─── Offres ───
        .route("/api/jobs", get(jobs::list_jobs))
        .route("/api/jobs/:id", get(jobs::get_job))
        // ─── Candidatures ───
        .route("/api/applications", post(applications::create_application))
        .route(
            "/api/applications/:id",
            patch(applications::update_application)
                .delete(applications::delete_application)
                .post(applications::update_application),
        )
        .route(
            "/api/applications/:id/delete",
            delete(applications::delete_application),
        )
}
