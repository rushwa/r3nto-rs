// crates/api/src/handlers/properties.rs
use axum::{extract::{State, Path}, Json};
use std::sync::Arc;
use uuid::Uuid;
use rento_core::{error::Result};
use crate::state::AppState;
use crate::middleware::auth::{RequireAuth, RequirePropertyOwner};

pub async fn list_properties(State(_state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "List properties - implement me"})))
}

pub async fn create_property(State(_state): State<Arc<AppState>>, _auth: RequirePropertyOwner, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Create property - implement me"})))
}

pub async fn get_my_properties(State(_state): State<Arc<AppState>>, _auth: RequireAuth) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get my properties - implement me"})))
}

pub async fn get_property(State(_state): State<Arc<AppState>>, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get property - implement me"})))
}

pub async fn update_property(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Update property - implement me"})))
}

pub async fn delete_property(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<axum::http::StatusCode> {
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn activate_property(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Activate property - implement me"})))
}

pub async fn deactivate_property(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Deactivate property - implement me"})))
}

pub async fn add_unit(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Add unit - implement me"})))
}

pub async fn add_images(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Add images - implement me"})))
}

pub async fn get_subscription_info(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get subscription info - implement me"})))
}

pub async fn check_free_trial(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Check free trial - implement me"})))
}

pub async fn get_user_limits(State(_state): State<Arc<AppState>>, _auth: RequireAuth) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get user limits - implement me"})))
}
