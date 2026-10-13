use axum::extract::State;
use axum::Json;
use chrono::Utc;

use crate::auth::{create_token, hash_password, verify_password, AuthUser};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User};
use crate::state::AppState;

/// POST /api/auth/register — création de compte.
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    let email = req.email.trim().to_lowercase();
    let full_name = req.full_name.trim().to_string();

    if full_name.is_empty() {
        return Err(AppError::BadRequest("le nom complet est requis".into()));
    }
    if !email.contains('@') || email.len() < 5 {
        return Err(AppError::BadRequest("adresse email invalide".into()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest(
            "le mot de passe doit contenir au moins 6 caractères".into(),
        ));
    }
    if db::email_exists(&state.pool, &email).await? {
        return Err(AppError::Conflict("cette adresse email est déjà utilisée".into()));
    }

    let hash = hash_password(&req.password).map_err(AppError::Internal)?;
    let id = uuid::Uuid::new_v4().to_string();
    let user = db::insert_user(&state.pool, &id, &email, &hash, &full_name).await?;
    let token = create_token(&user.id, &state.config.jwt_secret).map_err(AppError::Internal)?;

    tracing::info!(email = %email, "nouvel utilisateur inscrit");
    Ok(Json(AuthResponse { user, token }))
}

/// POST /api/auth/login — connexion.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let email = req.email.trim().to_lowercase();
    let found = db::get_user_with_hash(&state.pool, &email).await?;

    let Some((user, hash)) = found else {
        return Err(AppError::Unauthorized("email ou mot de passe incorrect".into()));
    };
    if !verify_password(&req.password, &hash) {
        return Err(AppError::Unauthorized("email ou mot de passe incorrect".into()));
    }

    let token = create_token(&user.id, &state.config.jwt_secret).map_err(AppError::Internal)?;
    Ok(Json(AuthResponse { user, token }))
}

/// GET /api/auth/me — utilisateur courant.
pub async fn me(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<User>> {
    db::get_user(&state.pool, &auth.user_id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

/// POST /api/auth/logout — déconnexion (côté client : supprimer le token).
pub async fn logout() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "message": "déconnecté" }))
}

/// GET /api/health authentifié — petite route de test protégée.
pub async fn protected_ping(auth: AuthUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "user_id": auth.user_id, "at": Utc::now() }))
}
