// crates/api/src/handlers/units.rs
use axum::{extract::{State, Path}, Json};
use std::sync::Arc;
use uuid::Uuid;
use rento_core::{error::Result};
use crate::state::AppState;
use crate::middleware::auth::RequireAuth;

pub async fn list_units(State(_state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "List units - implement me"})))
}

pub async fn get_unit(State(_state): State<Arc<AppState>>, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get unit - implement me"})))
}

pub async fn update_unit(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Update unit - implement me"})))
}

pub async fn delete_unit(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<axum::http::StatusCode> {
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn activate_unit(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Activate unit - implement me"})))
}

pub async fn deactivate_unit(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Deactivate unit - implement me"})))
}

pub async fn add_images(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Add unit images - implement me"})))
}

pub async fn get_my_units(State(_state): State<Arc<AppState>>, _auth: RequireAuth) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get my units - implement me"})))
}
