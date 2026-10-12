use std::sync::Arc;

use reqwest::Client;
use sqlx::SqlitePool;

use crate::agents::llm::LlmClient;
use crate::config::Config;

/// État partagé, injecté dans tous les handlers Axum.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Arc<Config>,
    pub llm: LlmClient,
    pub http: Client,
}

impl AppState {
    pub fn new(pool: SqlitePool, config: Config) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(config.scrape_timeout_secs))
            .connect_timeout(std::time::Duration::from_secs(8))
            .user_agent(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/124.0 Safari/537.36 Workeo/0.1",
            )
            .build()
            .expect("client HTTP");

        let llm = LlmClient::new(config.clone(), http.clone());
        AppState {
            pool,
            config: Arc::new(config),
            llm,
            http,
        }
    }
}
