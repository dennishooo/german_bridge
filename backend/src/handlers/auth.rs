use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use std::sync::Arc;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use crate::auth::{self, LoginRequest, RegisterRequest, AuthResponse};
use crate::server::AppState;
use crate::entities::user;
use uuid::Uuid;
use chrono::Utc;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 1. Check if user exists
    let existing_user = user::Entity::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing_user.is_some() {
        return Err((StatusCode::CONFLICT, "Username already exists".to_string()));
    }

    // 2. Hash password
    let password_hash = auth::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // 3. Create user
    let user_id = Uuid::new_v4();
    
    let new_user = user::ActiveModel {
        id: Set(user_id),
        username: Set(payload.username.clone()),
        password_hash: Set(password_hash),
        created_at: Set(Utc::now().into()),
    };
    
    new_user.insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. Generate Token
    let token = auth::create_jwt(&user_id.to_string(), &payload.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(AuthResponse {
        token,
        username: payload.username,
        user_id: user_id.to_string(),
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 1. Find user
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&state.db)
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
    let token = auth::create_jwt(&user.id.to_string(), &user.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(AuthResponse {
        token,
        username: user.username,
        user_id: user.id.to_string(),
    }))
}
