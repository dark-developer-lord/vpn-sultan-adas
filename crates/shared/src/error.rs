use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub enum AppError {
    // Auth errors
    InvalidCredentials,
    TokenExpired,
    InvalidToken,
    Unauthorized,
    Forbidden,

    // User errors
    UserNotFound,
    UserAlreadyExists,
    InvalidEmail,

    // Peer/Node errors
    PeerNotFound,
    NodeNotFound,
    PeerLimitExceeded,
    SubscriptionLimitExceeded,
    PeerAlreadyRevoked,
    NotFound,

    // DB errors
    DatabaseError(String),
    QueryError(String),

    // Crypto errors
    EncryptionError,
    DecryptionError,

    // Validation errors
    ValidationError(String),
    InvalidRequest(String),

    // Internal errors
    InternalServerError,
    NotImplemented,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidCredentials => write!(f, "Invalid email or password"),
            AppError::TokenExpired => write!(f, "Token has expired"),
            AppError::InvalidToken => write!(f, "Invalid token"),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Forbidden => write!(f, "Forbidden"),
            AppError::UserNotFound => write!(f, "User not found"),
            AppError::UserAlreadyExists => write!(f, "User already exists"),
            AppError::InvalidEmail => write!(f, "Invalid email format"),
            AppError::PeerNotFound => write!(f, "Peer not found"),
            AppError::NodeNotFound => write!(f, "Node not found"),
            AppError::PeerLimitExceeded => write!(f, "Peer limit exceeded for your subscription"),
            AppError::SubscriptionLimitExceeded => write!(f, "Subscription limit exceeded"),
            AppError::PeerAlreadyRevoked => write!(f, "Peer is already revoked"),
            AppError::NotFound => write!(f, "Resource not found"),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::QueryError(msg) => write!(f, "Query error: {}", msg),
            AppError::EncryptionError => write!(f, "Encryption error"),
            AppError::DecryptionError => write!(f, "Decryption error"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            AppError::InternalServerError => write!(f, "Internal server error"),
            AppError::NotImplemented => write!(f, "Not implemented"),
        }
    }
}

impl std::error::Error for AppError {}
