use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use std::sync::Arc;
use crate::auth::{self, LoginRequest, RegisterRequest, AuthResponse};
use crate::server::AppState;
use uuid::Uuid;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 1. Check if user exists
    let user_exists = sqlx::query!("SELECT id FROM users WHERE username = ?", payload.username)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if user_exists.is_some() {
        return Err((StatusCode::CONFLICT, "Username already exists".to_string()));
    }

    // 2. Hash password
    let password_hash = auth::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // 3. Create user
    let user_id = Uuid::new_v4().to_string();
    
    sqlx::query!(
        "INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)",
        user_id,
        payload.username,
        password_hash
    )
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. Generate Token
    let token = auth::create_jwt(&user_id, &payload.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(AuthResponse {
        token,
        username: payload.username,
        user_id,
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 1. Find user
    let user = sqlx::query!(
        "SELECT id, username, password_hash FROM users WHERE username = ?",
        payload.username
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = user.ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // 2. Verify password
    let valid = auth::verify_password(&payload.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    // 3. Generate Token
    let token = auth::create_jwt(&user.id, &user.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(AuthResponse {
        token,
        username: user.username,
        user_id: user.id,
    }))
}
