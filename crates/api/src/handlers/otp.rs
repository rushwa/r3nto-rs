// crates/api/src/handlers/otp.rs
use axum::{extract::State, Json};
use std::sync::Arc;
use rento_core::{error::Result};
use crate::state::AppState;

pub async fn request_email_otp(State(_state): State<Arc<AppState>>, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Request email OTP - implement me"})))
}

pub async fn confirm_email_otp(State(_state): State<Arc<AppState>>, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Confirm email OTP - implement me"})))
}

pub async fn request_whatsapp_otp(State(_state): State<Arc<AppState>>, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Request WhatsApp OTP - implement me"})))
}

pub async fn confirm_whatsapp_otp(State(_state): State<Arc<AppState>>, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Confirm WhatsApp OTP - implement me"})))
}
