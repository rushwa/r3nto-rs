// crates/api/src/handlers/images.rs
use axum::{extract::{State, Path}, Json};
use std::sync::Arc;
use uuid::Uuid;
use rento_core::{error::Result};
use crate::state::AppState;
use crate::middleware::auth::RequireAuth;

pub async fn set_main_property_image(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Set main property image - implement me"})))
}

pub async fn set_main_unit_image(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Set main unit image - implement me"})))
}
