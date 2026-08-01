// crates/api/src/handlers/property_owners.rs
use axum::{extract::State, Json};
use std::sync::Arc;
use rento_core::{error::Result};
use crate::state::AppState;
use crate::middleware::auth::{RequireAuth, RequireAgentOrAdmin};

pub async fn list_property_owners(State(_state): State<Arc<AppState>>, _auth: RequireAgentOrAdmin) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "List property owners - implement me"})))
}

pub async fn get_my_profile(State(_state): State<Arc<AppState>>, _auth: RequireAuth) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get my property owner profile - implement me"})))
}

pub async fn subscribe(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Subscribe - implement me"})))
}
