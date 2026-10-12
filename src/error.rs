use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Type d'erreur applicatif unique, convertible en réponse HTTP.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("ressource introuvable")]
    NotFound,

    #[error("requête invalide: {0}")]
    BadRequest(String),

    #[error("base de données: {0}")]
    Database(#[from] sqlx::Error),

    #[error("erreur réseau: {0}")]
    Http(#[from] reqwest::Error),

    #[error("erreur interne: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Database(e) => {
                tracing::error!("db error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données".into())
            }
            AppError::Http(e) => {
                tracing::warn!("http error: {e}");
                (StatusCode::BAD_GATEWAY, "source externe indisponible".into())
            }
            AppError::Internal(e) => {
                tracing::error!("internal error: {e:#}");
                (StatusCode::INTERNAL_SERVER_ERROR, "erreur interne".into())
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
