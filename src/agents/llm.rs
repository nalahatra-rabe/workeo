use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use crate::config::{Config, LlmProvider};

/// Client LLM unifié.
///
/// Le client n'échoue jamais de façon bloquante : en cas d'indisponibilité, il
/// renvoie `None` et les agents basculent sur leurs heuristiques internes.
/// Après un premier échec de connexion, il se désactive pour la session
/// (évite de re-tenter des appels coûteux à chaque requête).
#[derive(Clone)]
pub struct LlmClient {
    config: Config,
    http: reqwest::Client,
    /// Passé à `true` après un échec : court-circuite les appels suivants.
    degraded: Arc<AtomicBool>,
}

impl LlmClient {
    pub fn new(config: Config, http: reqwest::Client) -> Self {
        LlmClient {
            config,
            http,
            degraded: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Indique si un LLM est configuré et non dégradé.
    pub fn is_enabled(&self) -> bool {
        self.config.llm_provider != LlmProvider::None && !self.degraded.load(Ordering::Relaxed)
    }

    pub fn provider_label(&self) -> &'static str {
        self.config.llm_provider.label()
    }

    pub fn model_label(&self) -> String {
        match self.config.llm_provider {
            LlmProvider::Ollama => self.config.ollama_model.clone(),
            LlmProvider::OpenAi => self.config.openai_model.clone(),
            LlmProvider::None => "heuristiques".into(),
        }
    }

    /// Vérifie la disponibilité effective du LLM (ping léger).
    pub async fn health(&self) -> bool {
        match self.config.llm_provider {
            LlmProvider::None => false,
            LlmProvider::Ollama => {
                let url = format!("{}/api/tags", self.config.ollama_base_url.trim_end_matches('/'));
                self.http
                    .get(url)
                    .timeout(Duration::from_secs(3))
                    .send()
                    .await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false)
            }
            LlmProvider::OpenAi => !self.config.openai_api_key.is_empty(),
        }
    }

    /// Demande une réponse JSON au LLM. Renvoie `None` en cas d'échec/indispo.
    pub async fn complete_json(&self, system: &str, user: &str) -> Option<Value> {
        let raw = match self.config.llm_provider {
            LlmProvider::None => return None,
            LlmProvider::Ollama => self.call_ollama(system, user).await,
            LlmProvider::OpenAi => self.call_openai(system, user).await,
        };

        match raw {
            Some(text) => extract_json(&text),
            None => None,
        }
    }

    async fn call_ollama(&self, system: &str, user: &str) -> Option<String> {
        let url = format!(
            "{}/api/chat",
            self.config.ollama_base_url.trim_end_matches('/')
        );
        let body = json!({
            "model": self.config.ollama_model,
            "stream": false,
            "format": "json",
            "options": { "temperature": 0.2 },
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });

        let resp = match self
            .http
            .post(url)
            .timeout(Duration::from_secs(120))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("ollama injoignable ({e}) — bascule en mode heuristique");
                self.degraded.store(true, Ordering::Relaxed);
                return None;
            }
        };

        if !resp.status().is_success() {
            tracing::warn!("ollama statut {} — bascule en mode heuristique", resp.status());
            self.degraded.store(true, Ordering::Relaxed);
            return None;
        }
        let v: Value = resp.json().await.ok()?;
        v.get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
    }

    async fn call_openai(&self, system: &str, user: &str) -> Option<String> {
        if self.config.openai_api_key.is_empty() {
            return None;
        }
        let url = format!(
            "{}/chat/completions",
            self.config.openai_base_url.trim_end_matches('/')
        );
        let body = json!({
            "model": self.config.openai_model,
            "temperature": 0.2,
            "response_format": { "type": "json_object" },
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });

        let resp = match self
            .http
            .post(url)
            .bearer_auth(&self.config.openai_api_key)
            .timeout(Duration::from_secs(120))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("openai injoignable ({e}) — bascule en mode heuristique");
                self.degraded.store(true, Ordering::Relaxed);
                return None;
            }
        };

        if !resp.status().is_success() {
            tracing::warn!("openai statut {}", resp.status());
            self.degraded.store(true, Ordering::Relaxed);
            return None;
        }
        let v: Value = resp.json().await.ok()?;
        v.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
    }
}

/// Extrait le premier objet JSON valide d'une réponse texte (tolère ```json ...```).
fn extract_json(text: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(text.trim()) {
        return Some(v);
    }
    // Retire les fences markdown éventuelles.
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if let Ok(v) = serde_json::from_str::<Value>(cleaned) {
        return Some(v);
    }
    // Cherche la première { ... } équilibrée.
    let start = text.find('{')?;
    let mut depth = 0i32;
    for (i, ch) in text[start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let candidate = &text[start..start + i + 1];
                    return serde_json::from_str(candidate).ok();
                }
            }
            _ => {}
        }
    }
    None
}
