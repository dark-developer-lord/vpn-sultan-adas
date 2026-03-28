use axum::{http::StatusCode, Json};
use serde_json::json;
use vpn_shared::AppError;

pub fn map_error_to_response(error: AppError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, message) = match error {
        AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
        AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
        AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
        AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
        AppError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
        AppError::InvalidEmail => (StatusCode::BAD_REQUEST, "Invalid email format"),
        AppError::ValidationError(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
    };

    let body = json!({
        "status": "error",
        "error": message,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    (status, Json(body))
}
