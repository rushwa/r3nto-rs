use axum::{
    extract::State,
    Extension, Json,
};
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};
use crate::models::admin::Claims;
use crate::models::mpesa::{
    InitiatePaymentRequest, MpesaCallbackPayload, PaymentResponse,
};
use crate::services::{commissions, mpesa, wallet};
use crate::state::AppState;

// ───────────────────────────────────────────
// POST /admin/mpesa/stk-push
// Initiate a payment (simulation mode)
// ───────────────────────────────────────────
pub async fn initiate_payment(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<InitiatePaymentRequest>,
) -> ApiResult<Json<PaymentResponse>> {
    let payer_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let response = commissions::initiate_payment(
        &state.db.pool,
        &state.email,
        &state.mpesa,
        &payer_id,
        &req.phone_number,
        req.amount,
        &req.payment_type,
        req.reference_id.as_deref(), // ✅ FIX: Pass Option<&str> directly, let commissions.rs handle Uuid parsing
        &req.account_reference,
    ).await?;

    Ok(Json(response))
}

// ───────────────────────────────────────────
// POST /api/mpesa/callback
// Safaricom Daraja webhook (no auth, IP whitelisted)
// ───────────────────────────────────────────
pub async fn mpesa_callback(
    State(state): State<AppState>,
    Json(payload): Json<MpesaCallbackPayload>,
) -> ApiResult<Json<serde_json::Value>> {
    let callback = &payload.body.stk_callback;

    tracing::info!(
        "📥 M-Pesa callback received: checkout={}, result_code={}",
        callback.checkout_request_id, callback.result_code
    );

    Ok(Json(serde_json::json!({
        "ResultCode": 0,
        "ResultDesc": "Service processed successfully (simulation mode)"
    })))
}

// ───────────────────────────────────────────
// GET /admin/commissions/my
// ───────────────────────────────────────────
pub async fn get_my_commissions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<crate::models::mpesa::CommissionLedgerEntry>>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let entries = commissions::get_agent_commissions(&state.db.pool, &agent_id).await?;
    Ok(Json(entries))
}

// ───────────────────────────────────────────
// GET /admin/commissions/my/wallet
// ───────────────────────────────────────────
pub async fn get_my_wallet(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let wallet_info = wallet::get_or_create_wallet(&state.db.pool, &agent_id).await?;
    let history = wallet::get_wallet_history(&state.db.pool, &agent_id, 50).await?;

    Ok(Json(serde_json::json!({
        "wallet": wallet_info,
        "transactions": history,
    })))
}

// ───────────────────────────────────────────
// POST /admin/commissions/my/payout
// ───────────────────────────────────────────
pub async fn request_payout(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<crate::models::mpesa::PayoutRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid user ID: {}", e)))?;

    if req.amount <= 0.0 {
        return Err(ApiError::BadRequest("Amount must be positive".into()));
    }
    if req.amount < 50.0 {
        return Err(ApiError::BadRequest("Minimum payout is KES 50".into()));
    }

    let phone = mpesa::normalize_phone(&req.mpesa_phone)?;

    let wallet_info = wallet::get_or_create_wallet(&state.db.pool, &agent_id).await?;
    if wallet_info.balance < req.amount {
        return Err(ApiError::BadRequest(
            format!("Insufficient balance. Available: KES {:.2}", wallet_info.balance)
        ));
    }

    let wallet_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM agent_wallets WHERE agent_id = $1"
    )
        .bind(agent_id)
        .fetch_one(&state.db.pool)
        .await?;

    let payout_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO payout_requests (agent_id, wallet_id, amount, mpesa_phone)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#
    )
        .bind(agent_id)
        .bind(wallet_id)
        .bind(req.amount)
        .bind(&phone)
        .fetch_one(&state.db.pool)
        .await?;

    let mut tx = state.db.pool.begin().await?;
    wallet::debit_wallet(
        &mut tx,
        &agent_id,
        req.amount,
        &payout_id.to_string(),
        &format!("Payout request to M-Pesa {}", phone),
    ).await?;
    tx.commit().await?;

    sqlx::query("UPDATE agent_wallets SET mpesa_phone = $1 WHERE id = $2")
        .bind(&phone)
        .bind(wallet_id)
        .execute(&state.db.pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Payout request submitted. Awaiting admin approval.",
        "payout_id": payout_id.to_string(),
        "amount": req.amount,
        "phone": phone,
    })))
}