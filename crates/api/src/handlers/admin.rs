use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};
use crate::models::admin::{AdminUser, Claims, CreateUserRequest, LoginRequest, LoginResponse};
use crate::models::user::User;
use crate::models::agent::Agent;
use crate::models::property::{Property, PropertyDetail};
use crate::models::subscription::SubscriptionPlan;
use crate::models::commission::Commission;
use crate::models::inquiry::Inquiry;
use crate::models::analytics::{StatsData, SalesData, TopAgent, MarketTrend, SystemSettings};
use crate::services::admin as admin_service;
use crate::state::AppState;

// ───────────────────────────────────────────
// Request DTOs
// ───────────────────────────────────────────

// ... (keep existing imports and other handlers) ...

// ───────────────────────────────────────────
// Request DTOs (UPDATED: Requires BOTH UUID and Email)
// ───────────────────────────────────────────
#[derive(Deserialize)]
pub struct HandshakeInitiateRequest {
    pub target_user_id: String, // UUID
    pub target_email: String,   // Email
}

#[derive(Deserialize)]
pub struct HandshakeVerifyRequest {
    pub target_user_id: String, // UUID
    pub target_email: String,   // Email
    pub otp_code: String,
}

// ... (keep other handlers) ...

// ───────────────────────────────────────────
// Digital Handshake Handlers
// ───────────────────────────────────────────
pub async fn initiate_handshake(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<HandshakeInitiateRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if claims.role.to_uppercase() != "AGENT" && claims.role.to_uppercase() != "ADMIN" {
        return Err(ApiError::Unauthorized("Only Agents or Admins can initiate handshakes".to_string()));
    }

    admin_service::initiate_handshake(
        &state.db,
        &state.email,
        &claims.sub,
        &req.target_user_id,
        &req.target_email,
    ).await?;

    Ok(Json(serde_json::json!({
        "message": "Handshake OTP sent successfully to the owner's email."
    })))
}

pub async fn verify_handshake(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<HandshakeVerifyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if claims.role.to_uppercase() != "AGENT" && claims.role.to_uppercase() != "ADMIN" {
        return Err(ApiError::Unauthorized("Only Agents or Admins can verify handshakes".to_string()));
    }

    admin_service::verify_handshake(
        &state.db,
        &claims.sub,
        &req.target_user_id,
        &req.target_email,
        &req.otp_code,
    ).await?;

    Ok(Json(serde_json::json!({
        "message": "Digital Handshake successful. User is now a Property Owner with dashboard access."
    })))
}

#[derive(Deserialize)]
pub struct CreatePropertyRequest {
    pub title: String,
    pub description: Option<String>,
    pub property_type: Option<String>,
    pub price: Option<f64>,
    pub county: Option<String>,
    pub location: Option<String>,
    pub plot_number: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub purpose: Option<String>,
    pub general_features: Option<serde_json::Value>,
    pub video_url: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Deserialize)]
pub struct ToggleActiveRequest {
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct UpdateInquiryRequest {
    pub status: String,
    pub assigned_to: Option<String>,
}

#[derive(Deserialize)]
pub struct GrantPrivilegesRequest {
    pub user_id: String,
    pub grant: bool,
}

// ───────────────────────────────────────────
// Auth Handlers
// ───────────────────────────────────────────

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
) -> ApiResult<Json<AdminUser>> {
    let admin = admin_service::get_current_admin(&state.db, &claims).await?;
    Ok(Json(admin))
}

// ───────────────────────────────────────────
// Dashboard & Stats
// ───────────────────────────────────────────

pub async fn get_stats(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<StatsData>> {
    let stats = admin_service::get_stats(&state.db).await?;
    Ok(Json(stats))
}

// ───────────────────────────────────────────
// User Management
// ───────────────────────────────────────────

pub async fn create_user(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<CreateUserRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    admin_service::create_user(&state.db, &req).await?;
    Ok(Json(serde_json::json!({ "message": "User created successfully" })))
}

pub async fn get_users(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<User>>> {
    let users = admin_service::get_users(&state.db).await?;
    Ok(Json(users))
}

pub async fn get_user_profile(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let profile = admin_service::get_user_profile(&state.db, &id).await?;
    Ok(Json(profile))
}

pub async fn toggle_user_active(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(req): Json<ToggleActiveRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    admin_service::toggle_user_active(&state.db, &id, req.is_active).await?;
    Ok(Json(serde_json::json!({ "message": "User status updated" })))
}

pub async fn grant_admin_privileges(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<GrantPrivilegesRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    admin_service::grant_admin_privileges(&state.db, &req.user_id, req.grant).await?;
    Ok(Json(serde_json::json!({ "message": "Privileges updated" })))
}

// ───────────────────────────────────────────
// Agents
// ───────────────────────────────────────────

pub async fn get_agents(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<Agent>>> {
    let agents = admin_service::get_agents(&state.db).await?;
    Ok(Json(agents))
}

// ───────────────────────────────────────────
// Digital Handshake
// ───────────────────────────────────────────


// ───────────────────────────────────────────
// Registration Fee Status
// ───────────────────────────────────────────

pub async fn get_registration_fee_status(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let has_paid = admin_service::has_paid_registration_fee(&state.db, &user_id).await?;

    let fee_amount: f64 = sqlx::query_scalar(
        "SELECT registration_fee::float8 FROM system_settings WHERE id = 1"
    )
        .fetch_one(&state.db.pool)
        .await
        .unwrap_or(1000.0);

    let message = if has_paid {
        "Registration fee paid. You can now create properties.".to_string()
    } else {
        format!(
            "Please pay the registration fee of KES {:.0} to activate your account and create properties.",
            fee_amount
        )
    };

    Ok(Json(serde_json::json!({
        "has_paid_registration_fee": has_paid,
        "registration_fee_amount": fee_amount,
        "message": message
    })))
}

// ───────────────────────────────────────────
// Properties
// ───────────────────────────────────────────

pub async fn get_properties(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<Property>>> {
    let properties = admin_service::get_properties(&state.db, &claims).await?;
    Ok(Json(properties))
}

pub async fn get_property_detail(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<PropertyDetail>> {
    let property = admin_service::get_property_detail(&state.db, &id).await?;
    Ok(Json(property))
}

pub async fn create_property(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePropertyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // 1. Verify user is a PROPERTY_OWNER
    let user_role: String = sqlx::query_scalar(
        "SELECT role::text FROM account_users WHERE id = $1"
    )
        .bind(user_id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;

    if user_role.to_uppercase() != "PROPERTY_OWNER" {
        return Err(ApiError::BadRequest(
            "Only PROPERTY_OWNERs can create properties".into()
        ));
    }

    // 2. Check if they've paid the registration fee (GATE)
    let has_paid = admin_service::has_paid_registration_fee(&state.db, &user_id).await?;
    if !has_paid {
        return Err(ApiError::BadRequest(
            "You must pay the registration fee (KES 1000) before creating properties. Please complete the payment first.".into()
        ));
    }

    // 3. Create the property
    let property_id = Uuid::new_v4();
    let property_type_enum = req.property_type.as_deref().unwrap_or("apartment");

    sqlx::query(
        r#"
        INSERT INTO properties
            (id, title, description, property_type, price, owner_id, status,
             county, location, plot_number, constituency, ward, purpose,
             general_features, video_url, latitude, longitude, is_active, subscription_status)
        VALUES ($1, $2, $3, $4::text::property_type, $5, $6, 'available',
                $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, TRUE, 'active')
        "#
    )
        .bind(property_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(property_type_enum)
        .bind(req.price)
        .bind(user_id)
        .bind(&req.county)
        .bind(&req.location)
        .bind(&req.plot_number)
        .bind(&req.constituency)
        .bind(&req.ward)
        .bind(&req.purpose)
        .bind(&req.general_features)
        .bind(&req.video_url)
        .bind(req.latitude)
        .bind(req.longitude)
        .execute(&state.db.pool)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create property: {}", e)))?;

    // 4. Update property_owner_profiles count
    sqlx::query(
        "UPDATE property_owner_profiles SET properties_owned = properties_owned + 1, updated_at = NOW() WHERE user_id = $1"
    )
        .bind(user_id)
        .execute(&state.db.pool)
        .await?;

    tracing::info!("✅ Property created: {} by owner {}", property_id, user_id);

    Ok(Json(serde_json::json!({
        "message": "Property created successfully",
        "property_id": property_id.to_string()
    })))
}

// ───────────────────────────────────────────
// Leads
// ───────────────────────────────────────────

pub async fn get_agent_leads(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let leads = admin_service::get_agent_leads(&state.db, &claims).await?;
    Ok(Json(leads))
}

// ───────────────────────────────────────────
// Subscriptions
// ───────────────────────────────────────────

pub async fn get_subscription_plans(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<SubscriptionPlan>>> {
    let plans = admin_service::get_subscription_plans(&state.db).await?;
    Ok(Json(plans))
}

// ───────────────────────────────────────────
// Commissions & Inquiries
// ───────────────────────────────────────────

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
    Ok(Json(serde_json::json!({ "message": "Inquiry updated" })))
}

// ───────────────────────────────────────────
// Analytics
// ───────────────────────────────────────────

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

// ───────────────────────────────────────────
// Settings
// ───────────────────────────────────────────

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
    Json(settings): Json<SystemSettings>,
) -> ApiResult<Json<serde_json::Value>> {
    admin_service::update_settings(&state.db, &settings).await?;
    Ok(Json(serde_json::json!({ "message": "Settings updated" })))
}

pub async fn get_property_owners_with_status(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let owners = admin_service::get_property_owners_with_status(&state.db).await?;
    Ok(Json(owners))
}