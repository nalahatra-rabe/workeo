use std::env;

/// Configuration globale de l'application, chargée depuis l'environnement.
#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub llm_provider: LlmProvider,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub openai_base_url: String,
    pub openai_api_key: String,
    pub openai_model: String,
    pub scrape_timeout_secs: u64,
    pub scrape_sources: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LlmProvider {
    Ollama,
    OpenAi,
    None,
}

impl LlmProvider {
    pub fn label(&self) -> &'static str {
        match self {
            LlmProvider::Ollama => "ollama",
            LlmProvider::OpenAi => "openai",
            LlmProvider::None => "none",
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        // Charge le .env s'il existe (ignore l'erreur sinon).
        let _ = dotenvy::dotenv();

        let llm_provider = match env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "ollama".into())
            .to_lowercase()
            .as_str()
        {
            "none" | "off" | "disabled" => LlmProvider::None,
            "openai" | "cloud" => LlmProvider::OpenAi,
            _ => LlmProvider::Ollama,
        };

        Config {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://data/workeo.db".into()),
            llm_provider,
            ollama_base_url: env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".into()),
            ollama_model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.1".into()),
            openai_base_url: env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
            openai_api_key: env::var("OPENAI_API_KEY").unwrap_or_default(),
            openai_model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into()),
            scrape_timeout_secs: env::var("SCRAPE_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(15),
            scrape_sources: env::var("SCRAPE_SOURCES")
                .unwrap_or_else(|_| "arbeitnow,remotive,remoteok,demo".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        }
    }
}
