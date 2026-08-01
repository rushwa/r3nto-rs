// crates/api/src/handlers/agents.rs
use axum::{extract::{State, Path}, Json};
use std::sync::Arc;
use uuid::Uuid;
use rento_core::error::Result;
use crate::state::AppState;
use crate::middleware::auth::{RequireAuth, RequireStaff, RequireAgentOrAdmin};

pub async fn list_agents(State(_state): State<Arc<AppState>>, _auth: RequireStaff) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "List agents - implement me"})))
}

pub async fn create_agent(State(_state): State<Arc<AppState>>, _auth: RequireStaff, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Create agent - implement me"})))
}

pub async fn get_my_profile(State(_state): State<Arc<AppState>>, _auth: RequireAuth) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get my agent profile - implement me"})))
}

pub async fn get_my_commissions(State(_state): State<Arc<AppState>>, _auth: RequireAuth) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get my commissions - implement me"})))
}

pub async fn get_my_property_owners(State(_state): State<Arc<AppState>>, _auth: RequireAuth) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get my property owners - implement me"})))
}

pub async fn get_agent_commissions(State(_state): State<Arc<AppState>>, _auth: RequireStaff, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get agent commissions - implement me"})))
}

pub async fn get_agent_property_owners(State(_state): State<Arc<AppState>>, _auth: RequireAgentOrAdmin, Path(_id): Path<Uuid>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get agent property owners - implement me"})))
}

pub async fn get_agent_stats(State(_state): State<Arc<AppState>>, _auth: RequireStaff) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Get agent stats - implement me"})))
}

pub async fn register_property_owner(State(_state): State<Arc<AppState>>, _auth: RequireAuth, Json(_req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"detail": "Register property owner - implement me"})))
}
