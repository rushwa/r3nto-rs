use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use uuid::Uuid;
use sqlx::Row;

use crate::errors::{ApiError, ApiResult};
use crate::models::mpesa::{InitiatePaymentRequest, PaymentResponse};
use crate::services::commissions;
use crate::state::AppState;

// ───────────────────────────────────────────
// Helper: Extract JWT and verify user is PROPERTY_OWNER
// ───────────────────────────────────────────
async fn verify_property_owner(
    state: &AppState,
    headers: &HeaderMap,
) -> ApiResult<Uuid> {
    // 1. Extract JWT from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing or invalid Authorization header".into()))?;

    // 2. Verify the token and get user claims
    let claims = state.auth.verify_token(token)
        .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))?;
    let claims_str = claims.sub.to_string();
    let payer_id = Uuid::parse_str(&claims_str)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // 3. Verify user is a PROPERTY_OWNER
    // FIX: Convert to String and cast in SQL to completely avoid sqlx Uuid type inference issues
    let payer_id_str = payer_id.to_string();

    let row = sqlx::query(
        "SELECT role::text as role FROM account_users WHERE id = $1::uuid"
    )
        .bind(&payer_id_str) // ✅ Binds as &str, Postgres safely casts to uuid
        .fetch_optional(&state.db.pool)
        .await?;

    let user_role: String = match row {
        Some(r) => r.get("role"),
        None => return Err(ApiError::NotFound("User not found".into())),
    };

    if user_role.to_uppercase() != "PROPERTY_OWNER" {
        return Err(ApiError::BadRequest(
            format!("Only PROPERTY_OWNERs can make this payment. Your role: {}", user_role)
        ));
    }

    Ok(payer_id)
}

// ───────────────────────────────────────────
// POST /api/payments/registration-fee
// ───────────────────────────────────────────
pub async fn pay_registration_fee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<InitiatePaymentRequest>,
) -> ApiResult<Json<PaymentResponse>> {
    let payer_id = verify_property_owner(&state, &headers).await?;

    let response = commissions::initiate_payment(
        &state.db.pool,
        &state.email,
        &state.mpesa,
        &payer_id,
        &req.phone_number,
        req.amount,
        "registration_fee",
        req.reference_id.as_deref(),
        &req.account_reference,
    ).await?;

    Ok(Json(response))
}

// ───────────────────────────────────────────
// POST /api/payments/subscription
// ───────────────────────────────────────────
pub async fn pay_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<InitiatePaymentRequest>,
) -> ApiResult<Json<PaymentResponse>> {
    let payer_id = verify_property_owner(&state, &headers).await?;

    let response = commissions::initiate_payment(
        &state.db.pool,
        &state.email,
        &state.mpesa,
        &payer_id,
        &req.phone_number,
        req.amount,
        "subscription",
        req.reference_id.as_deref(),
        &req.account_reference,
    ).await?;

    Ok(Json(response))
}