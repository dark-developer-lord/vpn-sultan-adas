use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use vpn_data::Database;
use vpn_domain::{AuthService, UserService};
use vpn_shared::types::ApiResponse;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub email: String,
    pub token: String,
}

pub async fn register(
    State(db): State<Arc<Database>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AuthResponse>>), (StatusCode, Json<serde_json::Value>)> {
    // Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "error": "Email and password are required"
            })),
        ));
    }

    if payload.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "error": "Password must be at least 6 characters"
            })),
        ));
    }

    // Hash password
    let auth_service = AuthService::new(
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string()),
    );

    let password_hash = auth_service
        .hash_password(&payload.password)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "error": "Failed to hash password"
                })),
            )
        })?;

    // Create user
    let user = UserService::create_user(&db.pool, &payload.email, &password_hash)
        .await
        .map_err(|e| {
            let (status, message) = match e {
                vpn_shared::AppError::UserAlreadyExists => (
                    StatusCode::CONFLICT,
                    "User already exists with this email",
                ),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user"),
            };
            (
                status,
                Json(serde_json::json!({
                    "status": "error",
                    "error": message
                })),
            )
        })?;

    // Generate JWT
    let token = auth_service
        .generate_jwt(user.id, &user.email, "user")
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "error": "Failed to generate token"
                })),
            )
        })?;

    tracing::info!("User registered: {}", user.email);

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(AuthResponse {
            user_id: user.id,
            email: user.email,
            token,
        })),
    ))
}

pub async fn login(
    State(db): State<Arc<Database>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, (StatusCode, Json<serde_json::Value>)> {
    // Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "error": "Email and password are required"
            })),
        ));
    }

    let auth_service = AuthService::new(
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string()),
    );

    // Find user by email (with password hash)
    let user_with_pass = vpn_data::repository::UserRepository::find_by_email_with_password(
        &db.pool,
        &payload.email,
    )
    .await
    .map_err(|_| {
        tracing::warn!("Login failed: user not found ({})", payload.email);
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "status": "error",
                "error": "Invalid credentials"
            })),
        )
    })?;

    // Verify password
    let is_valid = auth_service
        .verify_password(&payload.password, &user_with_pass.password_hash)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "error": "Failed to verify password"
                })),
            )
        })?;

    if !is_valid {
        tracing::warn!("Login failed: invalid password ({})", payload.email);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "status": "error",
                "error": "Invalid credentials"
            })),
        ));
    }

    // Generate JWT
    let token = auth_service
        .generate_jwt(user_with_pass.id, &user_with_pass.email, "user")
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "error": "Failed to generate token"
                })),
            )
        })?;

    tracing::info!("User logged in: {}", user_with_pass.email);

    Ok(Json(ApiResponse::ok(AuthResponse {
        user_id: user_with_pass.id,
        email: user_with_pass.email,
        token,
    })))
}
