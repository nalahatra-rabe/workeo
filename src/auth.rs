use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use rand_core::OsRng;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::error::AppError;
use crate::models::Claims;
use crate::state::AppState;

// ─── Mot de passe ───────────────────────────────────────────

/// Hache un mot de passe avec Argon2id.
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash mot de passe: {e}"))?;
    Ok(hash.to_string())
}

/// Vérifie un mot de passe contre son hash.
pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

// ─── JWT ────────────────────────────────────────────────────

const TOKEN_TTL_DAYS: i64 = 30;

/// Génère un JWT signé pour un utilisateur.
pub fn create_token(user_id: &str, secret: &str) -> anyhow::Result<String> {
    let exp = (Utc::now() + Duration::days(TOKEN_TTL_DAYS)).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("création token: {e}"))
}

/// Vérifie un JWT et renvoie ses claims.
pub fn verify_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| anyhow::anyhow!("token invalide: {e}"))?;
    Ok(data.claims)
}

// ─── Extracteur Axum ────────────────────────────────────────

/// Extracteur : identifie l'utilisateur courant via le header
/// `Authorization: Bearer <jwt>` (ou le cookie `workeo_token`).
pub struct AuthUser {
    pub user_id: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts)
            .ok_or_else(|| AppError::Unauthorized("authentification requise".into()))?;

        let claims = verify_token(&token, &state.config.jwt_secret)
            .map_err(|_| AppError::Unauthorized("session expirée ou invalide".into()))?;

        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}

fn extract_token(parts: &Parts) -> Option<String> {
    // 1) Header Authorization: Bearer xxx
    if let Some(h) = parts.headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(s) = h.to_str() {
            if let Some(t) = s.strip_prefix("Bearer ") {
                return Some(t.trim().to_string());
            }
        }
    }
    // 2) Cookie workeo_token
    if let Some(c) = parts.headers.get(axum::http::header::COOKIE) {
        if let Ok(s) = c.to_str() {
            for part in s.split(';') {
                let part = part.trim();
                if let Some(v) = part.strip_prefix("workeo_token=") {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}
