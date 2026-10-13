mod agents;
mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod scraper;
mod services;
mod state;

use axum::extract::DefaultBodyLimit;
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ─── Logs ───────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "workeo=info,tower_http=warn,sqlx=warn".into()
        }))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // ─── Config ─────────────────────────────────────────────
    let config = Config::from_env();
    tracing::info!(
        "Workeo démarrage — LLM: {} ({})",
        config.llm_provider.label(),
        config.ollama_model
    );

    // ─── Base de données ────────────────────────────────────
    let pool = db::init_pool(&config.database_url).await?;
    tracing::info!("Base de données prête ({})", config.database_url);

    let state = AppState::new(pool, config.clone());

    // ─── Front-end (SPA servie statiquement) ────────────────
    let web_dir = std::env::var("WEB_DIR").unwrap_or_else(|_| "web".into());
    let index = std::path::Path::new(&web_dir).join("index.html");
    let static_files = ServeDir::new(&web_dir).fallback(ServeFile::new(index));

    // ─── Application ────────────────────────────────────────
    let app = routes::api_router()
        .with_state(state)
        // Upload de CV : jusqu'à 16 Mo.
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .fallback_service(static_files);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("▶  http://localhost:{}", config.port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
