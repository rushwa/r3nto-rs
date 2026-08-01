// crates/api/src/handlers/commissions.rs
use axum::{extract::{State, Path}, Json};
use std::sync::Arc;
use uuid::Uuid;
use rento_core::{error::Result};
use crate::state::AppState;
use crate::middleware::auth::{RequireAuth, RequireStaff};

pub async fn list_commissions(State(_state): State<Arc<AppState>>, _auth: RequireAuth) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "List commissions - implement me"})))
}

pub async fn mark_paid(State(_state): State<Arc<AppState>>, _auth: RequireStaff, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Mark commission paid - implement me"})))
}
