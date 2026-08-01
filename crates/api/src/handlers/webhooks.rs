// crates/api/src/handlers/webhooks.rs
use axum::{extract::State, Json};
use std::sync::Arc;
use rento_core::error::Result;
use crate::state::AppState;

pub async fn infobip_webhook(State(_state): State<Arc<AppState>>, Json(_req): Json<serde_json::Value>) -> Result<axum::http::StatusCode> {
    Ok(axum::http::StatusCode::OK)
}
