use axum::{
    extract::{Path, State},
    Extension, Json,
};
use axum::http::StatusCode;
use crate::errors::{ApiError, ApiResult};
use crate::models::analytics::{MarketTrend, SalesData, StatsData, SystemSettings, TopAgent};
use crate::models::commission::Commission;
use crate::models::inquiry::{Inquiry, UpdateInquiryRequest};
use crate::models::property::{Property, PropertyDetail};
use crate::models::subscription::SubscriptionPlan;
use crate::models::user::User;
use crate::models::agent::Agent;
use crate::services::admin as admin_service;
use crate::state::AppState;

use crate::models::admin::{Claims, CreateUserRequest, LoginRequest, LoginResponse, ToggleUserActiveRequest, UpdateUserRoleRequest};


pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let response = admin_service::login(&state.db, &state.jwt_secret, req).await?;
    Ok(Json(response))
}

pub async fn get_current_admin(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<crate::models::admin::AdminUser>> {
    let user = admin_service::get_current_admin(&state.db, &claims).await?;
    Ok(Json(user))
}

pub async fn get_stats(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<StatsData>> {
    let stats = admin_service::get_stats(&state.db).await?;
    Ok(Json(stats))
}

pub async fn get_users(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<User>>> {
    let users = admin_service::get_users(&state.db).await?;
    Ok(Json(users))
}

pub async fn get_agents(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<Agent>>> {
    let agents = admin_service::get_agents(&state.db).await?;
    Ok(Json(agents))
}

pub async fn get_properties(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<Property>>> {
    let properties = admin_service::get_properties(&state.db).await?;
    Ok(Json(properties))
}

pub async fn get_property_detail(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<PropertyDetail>> {
    let detail = admin_service::get_property_detail(&state.db, &id).await?;
    Ok(Json(detail))
}

pub async fn get_subscription_plans(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<SubscriptionPlan>>> {
    let plans = admin_service::get_subscription_plans(&state.db).await?;
    Ok(Json(plans))
}

pub async fn get_commissions(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<Commission>>> {
    let commissions = admin_service::get_commissions(&state.db).await?;
    Ok(Json(commissions))
}

pub async fn get_inquiries(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<Inquiry>>> {
    let inquiries = admin_service::get_inquiries(&state.db).await?;
    Ok(Json(inquiries))
}

pub async fn update_inquiry_status(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(req): Json<UpdateInquiryRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    admin_service::update_inquiry_status(&state.db, &id, &req.status, req.assigned_to.as_deref()).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn get_sales_data(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<SalesData>>> {
    let data = admin_service::get_sales_data(&state.db).await?;
    Ok(Json(data))
}

pub async fn get_top_agents(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<TopAgent>>> {
    let agents = admin_service::get_top_agents(&state.db).await?;
    Ok(Json(agents))
}

pub async fn get_market_trends(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<MarketTrend>>> {
    let trends = admin_service::get_market_trends(&state.db).await?;
    Ok(Json(trends))
}

pub async fn get_settings(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<SystemSettings>> {
    let settings = admin_service::get_settings(&state.db).await?;
    Ok(Json(settings))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<SystemSettings>,
) -> ApiResult<Json<serde_json::Value>> {
    admin_service::update_settings(&state.db, &req).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}


pub async fn toggle_user_active(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<ToggleUserActiveRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    admin_service::toggle_user_active(&state.db, &req.user_id, req.is_active).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
pub async fn get_user_profile(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let profile = admin_service::get_user_profile(&state.db, &id).await?;
    Ok(Json(profile))
}

pub async fn grant_admin_privileges(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Only superuser can grant superuser status
    if req.is_superuser && claims.role != "superuser" {
        return Err(ApiError::Unauthorized("Only superuser can grant superuser status".to_string()));
    }

    admin_service::update_user_role(&state.db, &req.user_id, &req.role, req.is_superuser, req.is_staff).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn create_user(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<CreateUserRequest>,
) -> ApiResult<StatusCode> {
    admin_service::create_user(&state.db, &req).await?;
    Ok(StatusCode::CREATED)  // 201 with empty body
}