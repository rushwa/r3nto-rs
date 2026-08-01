// crates/core/src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RentoError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Password hash error: {0}")]
    PasswordHash(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Email error: {0}")]
    Email(String),

    #[error("SMS error: {0}")]
    Sms(String),

    #[error("WhatsApp error: {0}")]
    WhatsApp(String),
}

pub type Result<T> = std::result::Result<T, RentoError>;

impl axum::response::IntoResponse for RentoError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        let (status, message) = match &self {
            RentoError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            RentoError::Authorization(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            RentoError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            RentoError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            RentoError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            RentoError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            RentoError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            RentoError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            RentoError::PasswordHash(_) => (StatusCode::BAD_REQUEST, "Invalid password".to_string()),
            RentoError::Jwt(_) => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            RentoError::Email(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            RentoError::Sms(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            RentoError::WhatsApp(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        };

        let body = serde_json::json!({
            "detail": message,
            "error": format!("{}", self)
        });

        (status, axum::Json(body)).into_response()
    }
}
