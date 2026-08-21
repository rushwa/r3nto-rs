use axum::{
    extract::{Path, State,Multipart,Query},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;
use std::path::PathBuf;
use tokio::fs;
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};



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



pub async fn get_pending_payouts(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let payouts = admin_service::get_pending_payouts(&state.db).await?;
    Ok(Json(payouts))
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub plan_id: String,
    pub property_id: String,
}


#[derive(Deserialize)]
pub struct SubscribePropertyRequest {
    pub plan_id: String,
    pub property_id: String,
    pub phone_number: String,
}

pub async fn subscribe_property(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SubscribePropertyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let result = admin_service::subscribe_property(
        &state.db,
        &state.email,
        &state.mpesa,
        &user_id,
        &req.plan_id,
        &req.property_id,
        &req.phone_number,
    ).await?;

    Ok(Json(result))
}
pub async fn get_subscriptions_overview(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let overview = admin_service::get_subscriptions_overview(&state.db, &user_id).await?;
    Ok(Json(overview))
}
pub async fn get_my_subscriptions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let rows = sqlx::query(
        r#"
        SELECT
            ps.id::text, ps.property_id::text, ps.plan_id::text,
            ps.status::text, ps.amount_paid::float8,
            ps.start_date, ps.end_date,
            p.title as property_title,
            sp.name as plan_name, sp.tier::text as plan_tier, sp.price::float8 as plan_price
        FROM property_subscriptions ps
        JOIN properties p ON ps.property_id = p.id
        JOIN subscription_plans sp ON ps.plan_id = sp.id
        WHERE p.owner_id = $1
        ORDER BY ps.created_at DESC
        "#
    )
        .bind(user_id)
        .fetch_all(&state.db.pool)
        .await?;

    let subscriptions: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "property_id": row.try_get::<String, _>("property_id").unwrap_or_default(),
            "property_title": row.try_get::<String, _>("property_title").unwrap_or_default(),
            "plan_name": row.try_get::<String, _>("plan_name").unwrap_or_default(),
            "plan_tier": row.try_get::<String, _>("plan_tier").unwrap_or_default(),
            "plan_price": row.try_get::<f64, _>("plan_price").unwrap_or(0.0),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "amount_paid": row.try_get::<f64, _>("amount_paid").unwrap_or(0.0),
            "start_date": row.try_get::<chrono::DateTime<chrono::Utc>, _>("start_date")
                .map(|d| d.to_string()).unwrap_or_default(),
            "end_date": row.try_get::<chrono::DateTime<chrono::Utc>, _>("end_date")
                .map(|d| d.to_string()).unwrap_or_default(),
        })
    }).collect();

    Ok(Json(subscriptions))
}
pub async fn get_my_commissions_summary(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Get wallet info (includes ALL earnings: handshake + subscription + tour fees)
    let wallet = crate::services::wallet::get_or_create_wallet(&state.db.pool, &agent_id).await?;

    // Get recent commissions from ledger
    let recent_commissions = crate::services::commissions::get_agent_commissions(&state.db.pool, &agent_id).await?;

    // ✅ Get earnings breakdown by type
    let breakdown: Vec<(String, f64, i64)> = sqlx::query_as(
        r#"
        SELECT
            commission_type,
            COALESCE(SUM(commission_amount)::float8, 0) as total,
            COUNT(*) as count
        FROM commission_ledger
        WHERE agent_id = $1 AND status = 'credited'
        GROUP BY commission_type
        ORDER BY total DESC
        "#
    )
        .bind(agent_id)
        .fetch_all(&state.db.pool)
        .await?;

    let earnings_breakdown: Vec<serde_json::Value> = breakdown.iter().map(|(ctype, total, count)| {
        let label = match ctype.as_str() {
            "handshake_30pct" => "Registration Fee Commission (30%)",
            "subscription_10pct" => "Subscription Commission (10%)",
            "tour_fee" => "Virtual Tour Fee",
            "referral_bonus" => "Referral Bonus",
            other => other,
        };
        serde_json::json!({
            "type": ctype,
            "label": label,
            "total": total,
            "count": count,
        })
    }).collect();

    // ✅ Get tour-specific stats
    let tour_stats: Option<(i64, f64)> = sqlx::query_as(
        r#"
        SELECT COUNT(*), COALESCE(SUM(commission_amount)::float8, 0)
        FROM commission_ledger
        WHERE agent_id = $1 AND commission_type = 'tour_fee' AND status = 'credited'
        "#
    )
        .bind(agent_id)
        .fetch_one(&state.db.pool)
        .await
        .ok();

    let (tours_completed, tour_earnings) = tour_stats.unwrap_or((0, 0.0));

    Ok(Json(serde_json::json!({
        "wallet": {
            "balance": wallet.balance,
            "total_earned": wallet.total_earned,  // ✅ This includes ALL earnings
            "pending_balance": wallet.pending_balance,
            "total_withdrawn": wallet.total_withdrawn,
            "minimum_payout": 500.0,
            "can_request_payout": wallet.balance >= 500.0,
        },
        "earnings_breakdown": earnings_breakdown,
        "tour_earnings": {
            "tours_completed": tours_completed,
            "total_earned": tour_earnings,
            "fee_per_tour": 20.0,
        },
        "recent_commissions": recent_commissions.iter().take(10).map(|c| {
            serde_json::json!({
                "id": c.id,
                "type": c.commission_type,
                "amount": c.commission_amount,
                "gross_amount": c.gross_amount,
                "status": c.status,
                "created_at": c.created_at,
            })
        }).collect::<Vec<_>>(),
    })))
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
// Full User Role Management DTO
// ───────────────────────────────────────────
#[derive(Deserialize)]
pub struct UpdateUserRoleRequest {
    pub user_id: String,
    pub role: Option<String>,
    pub is_staff: Option<bool>,
    pub is_superuser: Option<bool>,
}

// ───────────────────────────────────────────
// Full User Role Management Handler
// ───────────────────────────────────────────
pub async fn update_user_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Only SUPERUSER can manage roles
    if claims.role.to_uppercase() != "SUPERUSER" && claims.role.to_uppercase() != "ADMIN" {
        return Err(ApiError::Unauthorized("Only admins can manage user roles".to_string()));
    }

    let user_id = Uuid::parse_str(&req.user_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let result = admin_service::update_user_role(
        &state.db,
        &user_id,
        req.role.as_deref(),
        req.is_staff,
        req.is_superuser,
    ).await?;

    Ok(Json(result))
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
#[derive(Deserialize)]
pub struct CreatePropertyRequest {
    pub id: Option<String>,
    pub title: String,
    pub description: Option<String>,

    // ✅ NEW: Purpose replaces price as the key property attribute
    pub purpose: String,           // "for_rent" | "for_sale" | "for_rent_and_sale"
    pub property_type: String,     // "apartment", "maisonette", "land", etc.
    pub status: Option<String>,

    // ✅ NEW: Land-specific fields
    pub is_land: Option<bool>,
    pub plot_size: Option<String>,
    pub plot_dimensions: Option<String>,
    pub land_price: Option<f64>,

    // Location (strings, not IDs)
    pub county: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub location: Option<String>,
    pub village: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub map_address: Option<String>,

    #[serde(default)]
    pub images: Vec<String>,
}
pub async fn create_or_update_property(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePropertyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let owner_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let result = admin_service::create_or_update_property(
        &state.db,
        &owner_id,
        &req,
    ).await?;

    Ok(Json(result))
}
// ───────────────────────────────────────────
// Leads
// ───────────────────────────────────────────

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

// ───────────────────────────────────────────
// Payment History
// ───────────────────────────────────────────
pub async fn get_payment_history(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let history = admin_service::get_payment_history(&state.db, &user_id).await?;
    Ok(Json(history))
}

pub async fn get_payment_summary(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let summary = admin_service::get_payment_summary(&state.db, &user_id).await?;
    Ok(Json(summary))
}
// ───────────────────────────────────────────
// Payout Request DTOs
// ───────────────────────────────────────────

#[derive(Deserialize)]
pub struct RequestPayoutRequest {
    pub amount: f64,
    pub mpesa_phone: String,
}

#[derive(Deserialize)]
pub struct PayoutActionRequest {
    pub payout_id: String,
    pub admin_notes: Option<String>,
}

// ───────────────────────────────────────────
// Agent: Request Payout
// ───────────────────────────────────────────
pub async fn request_payout(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<RequestPayoutRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if claims.role.to_uppercase() != "AGENT" {
        return Err(ApiError::Unauthorized("Only agents can request payouts".to_string()));
    }

    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let result = admin_service::request_payout(
        &state.db,
        &agent_id,
        req.amount,
        &req.mpesa_phone,
    ).await?;

    Ok(Json(result))
}

// ───────────────────────────────────────────
// Agent: Get Payout History
// ───────────────────────────────────────────
pub async fn get_my_payout_history(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let history = admin_service::get_agent_payout_history(&state.db, &agent_id).await?;
    Ok(Json(history))
}

// ───────────────────────────────────────────
// Admin: Get All Payout History
// ───────────────────────────────────────────
pub async fn get_all_payout_history(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    if claims.role.to_uppercase() != "ADMIN" && claims.role.to_uppercase() != "SUPERUSER" {
        return Err(ApiError::Unauthorized("Only admins can view payout history".to_string()));
    }

    let status = params.get("status").map(|s| s.as_str());
    let history = admin_service::get_all_payout_history(&state.db, status).await?;
    Ok(Json(history))
}

// ───────────────────────────────────────────
// Admin: Get Payout Stats
// ───────────────────────────────────────────
pub async fn get_payout_stats(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let stats = admin_service::get_payout_stats(&state.db).await?;
    Ok(Json(stats))
}

// ───────────────────────────────────────────
// Admin: Approve Payout (with email)
// ───────────────────────────────────────────
pub async fn approve_payout(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<PayoutActionRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if claims.role.to_uppercase() != "ADMIN" && claims.role.to_uppercase() != "SUPERUSER" {
        return Err(ApiError::Unauthorized("Only admins can approve payouts".to_string()));
    }

    let id = Uuid::parse_str(&req.payout_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // Get payout details
    let payout_info: Option<(Uuid, Uuid, f64, String, String, String)> = sqlx::query_as(
        r#"
        SELECT pr.id, pr.agent_id, pr.amount::float8, pr.status, pr.mpesa_phone,
               COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name
        FROM payout_requests pr
        JOIN account_users u ON pr.agent_id = u.id
        WHERE pr.id = $1
        "#
    )
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?;

    let (payout_uuid, agent_id, amount, status, phone, agent_name) = match payout_info {
        Some(p) => p,
        None => return Err(ApiError::NotFound("Payout not found".into())),
    };

    if status != "pending" {
        return Err(ApiError::BadRequest(format!("Payout is already {}", status)));
    }

    // Get agent email
    let agent_email: String = sqlx::query_scalar(
        "SELECT email FROM account_users WHERE id = $1"
    )
        .bind(agent_id)
        .fetch_one(&state.db.pool)
        .await?;

    // Update status
    sqlx::query(
        "UPDATE payout_requests SET status = 'approved', processed_at = NOW(), admin_notes = $2 WHERE id = $1"
    )
        .bind(id)
        .bind(&req.admin_notes)
        .execute(&state.db.pool)
        .await?;

    // Clear pending balance
    sqlx::query(
        "UPDATE agent_wallets SET pending_balance = GREATEST(0, pending_balance - $1), updated_at = NOW() WHERE agent_id = $2"
    )
        .bind(amount)
        .bind(agent_id)
        .execute(&state.db.pool)
        .await?;

    // Send email notification
    let _ = state.email.send_payout_approved(&agent_email, &agent_name, amount, &phone).await
        .map_err(|e| tracing::warn!("Failed to send payout approval email: {}", e));

    tracing::info!("✅ Payout {} approved for agent {} (KES {:.2})", payout_uuid, agent_name, amount);
    Ok(Json(serde_json::json!({ "message": "Payout approved and agent notified" })))
}

// ───────────────────────────────────────────
// Admin: Reject Payout (with email + refund)
// ───────────────────────────────────────────
pub async fn reject_payout(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<PayoutActionRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if claims.role.to_uppercase() != "ADMIN" && claims.role.to_uppercase() != "SUPERUSER" {
        return Err(ApiError::Unauthorized("Only admins can reject payouts".to_string()));
    }

    let id = Uuid::parse_str(&req.payout_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // Get payout details
    let payout_info: Option<(Uuid, Uuid, f64, String, String)> = sqlx::query_as(
        r#"
        SELECT pr.id, pr.agent_id, pr.amount::float8, pr.status,
               COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name
        FROM payout_requests pr
        JOIN account_users u ON pr.agent_id = u.id
        WHERE pr.id = $1
        "#
    )
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?;

    let (payout_uuid, agent_id, amount, status, agent_name) = match payout_info {
        Some(p) => p,
        None => return Err(ApiError::NotFound("Payout not found".into())),
    };

    if status != "pending" {
        return Err(ApiError::BadRequest(format!("Cannot reject payout with status: {}", status)));
    }

    // Get agent email
    let agent_email: String = sqlx::query_scalar(
        "SELECT email FROM account_users WHERE id = $1"
    )
        .bind(agent_id)
        .fetch_one(&state.db.pool)
        .await?;

    let mut tx = state.db.pool.begin().await?;

    // Refund wallet (credit back)
    crate::services::wallet::credit_wallet(
        &mut tx,
        &agent_id,
        amount,
        &payout_uuid.to_string(),
        &format!("Payout rejected - funds refunded"),
    ).await?;

    // Clear pending balance
    sqlx::query(
        "UPDATE agent_wallets SET pending_balance = GREATEST(0, pending_balance - $1), updated_at = NOW() WHERE agent_id = $2"
    )
        .bind(amount)
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    // Mark as rejected
    sqlx::query(
        "UPDATE payout_requests SET status = 'rejected', processed_at = NOW(), admin_notes = $2 WHERE id = $1"
    )
        .bind(id)
        .bind(&req.admin_notes)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    // Send email notification
    let _ = state.email.send_payout_rejected(&agent_email, &agent_name, amount).await
        .map_err(|e| tracing::warn!("Failed to send payout rejection email: {}", e));

    tracing::info!("❌ Payout {} rejected, KES {:.2} refunded to agent {}", req.payout_id, amount, agent_name);
    Ok(Json(serde_json::json!({ "message": "Payout rejected, funds refunded, and agent notified" })))
}
// ───────────────────────────────────────────
// Owner Inquiry Handlers
// ───────────────────────────────────────────

pub async fn get_owner_inquiries(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let inquiries = admin_service::get_owner_inquiries(&state.db, &user_id).await?;
    Ok(Json(inquiries))
}

pub async fn update_owner_inquiry_status(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let new_status = req.get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("Missing 'status' field".into()))?;

    admin_service::update_owner_inquiry_status(&state.db, &user_id, &id, new_status).await?;
    Ok(Json(serde_json::json!({ "message": "Inquiry status updated" })))
}
// ───────────────────────────────────────────
// Agent Lead Handlers
// ───────────────────────────────────────────

pub async fn get_agent_leads(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let leads = admin_service::get_agent_leads(&state.db, &agent_id).await?;
    Ok(Json(leads))
}

pub async fn update_lead_stage(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let new_stage = req.get("pipeline_stage")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("Missing 'pipeline_stage' field".into()))?;

    admin_service::update_lead_stage(&state.db, &agent_id, &id, new_stage).await?;
    Ok(Json(serde_json::json!({ "message": "Lead stage updated successfully" })))
}

// ───────────────────────────────────────────
// Agent Performance
// ───────────────────────────────────────────
pub async fn get_agent_performance(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let performance = admin_service::get_agent_performance(&state.db, &agent_id).await?;
    Ok(Json(performance))
}

// ───────────────────────────────────────────
// Agent Referrals
// ───────────────────────────────────────────
pub async fn get_agent_referrals(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let referrals = admin_service::get_agent_referrals(&state.db, &agent_id).await?;
    Ok(Json(referrals))
}

#[derive(Deserialize)]
pub struct RecordReferralRequest {
    pub referred_email: String,
    pub referred_name: Option<String>,
}

pub async fn record_referral(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<RecordReferralRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let result = admin_service::record_referral_signup(
        &state.db,
        &agent_id,
        &req.referred_email,
        req.referred_name.as_deref(),
    ).await?;
    Ok(Json(result))
}

// ───────────────────────────────────────────
// B2C Payout Processing
// ───────────────────────────────────────────
pub async fn process_b2c_payout(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(payout_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if claims.role.to_uppercase() != "ADMIN" && claims.role.to_uppercase() != "SUPERUSER" {
        return Err(ApiError::Unauthorized("Only admins can process B2C payouts".to_string()));
    }

    let result = admin_service::process_approved_payout_b2c(&state.db, &payout_id).await?;
    Ok(Json(result))
}

pub async fn get_b2c_history(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let history = admin_service::get_b2c_payout_history(&state.db, &agent_id).await?;
    Ok(Json(history))
}

// ───────────────────────────────────────────
// Bonus Tiers
// ───────────────────────────────────────────
pub async fn get_bonus_tiers(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let tiers = admin_service::get_bonus_tiers(&state.db).await?;
    Ok(Json(tiers))
}

pub async fn get_my_bonus_progress(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let progress = admin_service::get_agent_bonus_progress(&state.db, &agent_id).await?;
    Ok(Json(progress))
}

pub async fn claim_bonus(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let awarded = admin_service::check_and_award_bonuses(&state.db, &agent_id).await?;

    if awarded.is_empty() {
        Ok(Json(serde_json::json!({ "message": "No new bonuses to claim", "awarded": [] })))
    } else {
        Ok(Json(serde_json::json!({
            "message": format!("🏆 {} bonus(es) awarded!", awarded.len()),
            "awarded": awarded,
        })))
    }
}

// ───────────────────────────────────────────
// Leaderboard
// ───────────────────────────────────────────
pub async fn get_leaderboard(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let role = claims.role.to_uppercase();
    let agent_id = Uuid::parse_str(&claims.sub).ok();

    // Agents see top 20 + their own rank; Admins see top 50
    let limit = if role == "ADMIN" || role == "SUPERUSER" { 50 } else { 20 };
    let current_id = if role == "AGENT" { agent_id.as_ref() } else { None };

    let leaderboard = admin_service::get_leaderboard(&state.db, current_id, limit).await?;
    Ok(Json(leaderboard))
}


// ═══════════════════════════════════════════
// Virtual Tour Handlers (UPDATED with email)
// ═══════════════════════════════════════════

#[derive(Deserialize)]
pub struct RequestTourRequest {
    pub property_id: String,
    pub client_email: String,
    pub client_name: Option<String>,
    pub client_phone: Option<String>,
}

// ✅ UPDATED: Passes email service
pub async fn request_virtual_tour(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<RequestTourRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let client_id = Uuid::parse_str(&claims.sub).ok();
    let result = admin_service::request_virtual_tour(
        &state.db,
        &state.email,  // ✅ PASS EMAIL SERVICE
        &req.property_id,
        &req.client_email,
        req.client_name.as_deref(),
        req.client_phone.as_deref(),
        client_id.as_ref(),
    ).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct ConfirmTourPaymentRequest {
    pub payment_reference: String,
}

// ✅ UPDATED: Passes email service
pub async fn confirm_tour_payment(
    State(state): State<AppState>,
    Path(request_id): Path<String>,
    Json(req): Json<ConfirmTourPaymentRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let result = admin_service::confirm_tour_payment(
        &state.db,
        &state.email,  // ✅ PASS EMAIL SERVICE
        &request_id,
        &req.payment_reference,
    ).await?;
    Ok(Json(result))
}

pub async fn upload_tour_video(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Collect form fields and file
    let mut tour_request_id: Option<String> = None;
    let mut duration_seconds: Option<i32> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_mime: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Multipart error: {}", e);
        ApiError::BadRequest(format!("Invalid upload: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "tour_request_id" => {
                let text = field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read tour_request_id: {}", e))
                })?;
                tour_request_id = Some(text);
            }
            "duration_seconds" => {
                let text = field.text().await.map_err(|_| {
                    ApiError::BadRequest("Failed to read duration".into())
                })?;
                duration_seconds = text.parse::<i32>().ok();
            }
            "video" => {
                let mime = field.content_type().map(|s| s.to_string());
                let data = field.bytes().await.map_err(|e| {
                    tracing::error!("Failed to read video bytes: {}", e);
                    ApiError::BadRequest(format!("Failed to read video: {}", e))
                })?;
                file_data = Some(data.to_vec());
                file_mime = mime;
            }
            _ => {}
        }
    }

    // Validate required fields
    let tour_id_str = tour_request_id.ok_or_else(|| {
        ApiError::BadRequest("Missing tour_request_id".into())
    })?;

    let video_bytes = file_data.ok_or_else(|| {
        ApiError::BadRequest("Missing video file".into())
    })?;

    if video_bytes.is_empty() {
        return Err(ApiError::BadRequest("Video file is empty".into()));
    }

    // Generate unique filename
    let file_id = Uuid::new_v4();
    let extension = file_mime
        .as_deref()
        .and_then(|m| m.split('/').nth(1))
        .unwrap_or("webm");
    let filename = format!("{}.{}", file_id, extension);
    let file_path = PathBuf::from("uploads/tours").join(&filename);

    // Ensure directory exists
    fs::create_dir_all("uploads/tours").await.map_err(|e| {
        tracing::error!("Failed to create uploads directory: {}", e);
        ApiError::Internal("Failed to prepare storage".into())
    })?;

    // Write file to disk
    fs::write(&file_path, &video_bytes).await.map_err(|e| {
        tracing::error!("Failed to write video file: {}", e);
        ApiError::Internal("Failed to save video".into())
    })?;

    let file_size = video_bytes.len() as i64;
    let video_url = format!("/uploads/tours/{}", filename);

    tracing::info!(
        "🎬 Video saved: {} ({} bytes) for tour {}",
        video_url, file_size, tour_id_str
    );

    let upload_req = admin_service::UploadTourVideoRequest {
        tour_request_id: tour_id_str,
        video_url,
        thumbnail_url: None,
        duration_seconds,
        file_size_bytes: Some(file_size),
        device_fingerprint: None,
        recording_started_at: None,
        recording_completed_at: None,
    };

    // ✅ Get viewing link base from env
    let viewing_link_base = std::env::var("CLIENT_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());

    // ✅ Call service with email_service + viewing_link_base
    let result = admin_service::upload_tour_video(
        &state.db,
        &state.email,
        &viewing_link_base,
        &agent_id,
        &upload_req,
    ).await?;

    Ok(Json(result))
}
// Viewing link handler (unchanged - used for manual share button)
pub async fn generate_viewing_link(
    State(state): State<AppState>,
    Path(request_id): Path<String>,
    Extension(claims): Extension<Claims>,
) -> Response {
    let client_id = Uuid::parse_str(&claims.sub).ok();
    match admin_service::generate_viewing_link(&state.db, &request_id, client_id).await {
        Ok(result) => axum::Json(result).into_response(),
        Err(e) => {
            let error_detail = format!("{:?}", e);
            tracing::error!("❌ generate_viewing_link FAILED: {}", error_detail);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("ERROR: {}", error_detail),
            ).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct AccessTourRequest {
    pub device_fingerprint: String,
}

pub async fn access_tour_video(
    State(state): State<AppState>,
    Path(viewing_token): Path<String>,
    Json(req): Json<AccessTourRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let result = admin_service::access_tour_video(
        &state.db,
        &viewing_token,
        &req.device_fingerprint,
        None,
        None,
    ).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct DelistPropertyRequest {
    pub reason: Option<String>,
}

pub async fn delist_property(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(property_id): Path<String>,
    Json(req): Json<DelistPropertyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;
    let result = admin_service::delist_property(&state.db, &agent_id, &property_id, req.reason.as_deref()).await?;
    Ok(Json(result))
}

pub async fn get_agent_pending_tours(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;
    let tours = admin_service::get_agent_pending_tours(&state.db, &agent_id).await?;
    Ok(Json(tours))
}

#[derive(Deserialize)]
pub struct TourHistoryQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get_agent_tour_history(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<TourHistoryQuery>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;
    let history = admin_service::get_agent_tour_history(
        &state.db,
        &agent_id,
        params.status.as_deref(),
        params.limit,
    ).await?;
    Ok(Json(history))
}

pub async fn get_agent_sla_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;
    let stats = admin_service::get_agent_sla_stats(&state.db, &agent_id).await?;
    Ok(Json(stats))
}

#[derive(Deserialize)]
pub struct StreamQuery {
    pub fp: String,
}

pub async fn stream_tour_video(
    State(state): State<AppState>,
    Path(viewing_token): Path<String>,
    Query(params): Query<StreamQuery>,
) -> Response {
    let file_path = match admin_service::validate_tour_stream_access(
        &state.db,
        &viewing_token,
        &params.fp,
    ).await {
        Ok(path) => path,
        Err(ApiError::NotFound(msg)) => {
            return (StatusCode::NOT_FOUND, msg).into_response();
        }
        Err(ApiError::BadRequest(msg)) => {
            return (StatusCode::GONE, msg).into_response();
        }
        Err(ApiError::Unauthorized(msg)) => {
            return (StatusCode::FORBIDDEN, msg).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response();
        }
    };

    let video_bytes = match tokio::fs::read(&file_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read video file {}: {}", file_path, e);
            return (StatusCode::NOT_FOUND, "Video file not found").into_response();
        }
    };

    let content_type = if file_path.ends_with(".mp4") {
        "video/mp4"
    } else {
        "video/webm"
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_LENGTH, &*video_bytes.len().to_string()),
            (header::CACHE_CONTROL, &*"no-store".to_string()),
        ],
        Body::from(video_bytes),
    ).into_response()
}

// ───────────────────────────────────────────
// Public Property Handlers (No Auth)
// ───────────────────────────────────────────
pub async fn get_public_properties(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<Property>>> {
    let properties = admin_service::get_public_properties(&state.db).await?;
    Ok(Json(properties))
}

pub async fn get_public_property_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<PropertyDetail>> {
    let property = admin_service::get_public_property_detail(&state.db, &id).await?;
    Ok(Json(property))
}

// Add at the bottom of handlers/admin.rs
pub async fn get_my_tours(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let client_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;
    let tours = admin_service::get_client_tours(&state.db, &client_id).await?;
    Ok(Json(tours))
}

// ───────────────────────────────────────────
// Admin Tour Oversight
// ───────────────────────────────────────────
#[derive(Deserialize)]
pub struct AdminTourQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get_all_tours_admin(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<AdminTourQuery>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let role = claims.role.to_uppercase();
    if role != "ADMIN" && role != "SUPERUSER" {
        return Err(ApiError::Unauthorized("Only admins can view all tours".into()));
    }
    let tours = admin_service::get_all_tours_admin(
        &state.db,
        params.status.as_deref(),
        params.limit,
    ).await?;
    Ok(Json(tours))
}

pub async fn get_tour_stats_admin(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let role = claims.role.to_uppercase();
    if role != "ADMIN" && role != "SUPERUSER" {
        return Err(ApiError::Unauthorized("Only admins can view tour stats".into()));
    }
    let stats = admin_service::get_tour_stats_admin(&state.db).await?;
    Ok(Json(stats))
}

// ───────────────────────────────────────────
// Location Hierarchy Handlers
// ───────────────────────────────────────────
pub async fn get_countries(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let countries = admin_service::get_countries(&state.db).await?;
    Ok(Json(countries))
}

pub async fn get_location_children(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(parent_id): Path<i32>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let children = admin_service::get_location_children(&state.db, parent_id).await?;
    Ok(Json(children))
}

pub async fn get_unit_features(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let features = admin_service::get_unit_features(&state.db).await?;
    Ok(Json(features))
}

// ───────────────────────────────────────────
// Property Units Handlers
// ───────────────────────────────────────────
#[derive(Deserialize)]
pub struct CreateUnitRequest {
    pub property_id: String,
    pub unit: serde_json::Value,
}

pub async fn create_property_unit(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateUnitRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let owner_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;
    let property_id = Uuid::parse_str(&req.property_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid property ID: {}", e)))?;

    let result = admin_service::create_property_unit(&state.db, &property_id, &owner_id, &req.unit).await?;
    Ok(Json(result))
}

pub async fn get_property_units(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(property_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let prop_id = Uuid::parse_str(&property_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid property ID: {}", e)))?;
    let units = admin_service::get_property_units(&state.db, &prop_id).await?;
    Ok(Json(units))
}

#[derive(Deserialize)]
pub struct UpdateGeolocationRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub map_address: Option<String>,
}

pub async fn update_geolocation(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(property_id): Path<String>,
    Json(req): Json<UpdateGeolocationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let owner_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;
    let prop_id = Uuid::parse_str(&property_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid property ID: {}", e)))?;

    admin_service::update_property_geolocation(
        &state.db,
        &prop_id,
        &owner_id,
        req.latitude,
        req.longitude,
        req.map_address.as_deref(),
    ).await?;

    Ok(Json(serde_json::json!({ "message": "Geolocation updated" })))
}
