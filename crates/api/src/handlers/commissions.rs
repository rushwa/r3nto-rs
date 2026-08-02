use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use rento_core::models::agent::{Commission, MpesaCallbackRequest};
use rento_core::models::user::AccountUser;

pub async fn mpesa_callback(
    State(pool): State<PgPool>,
    Json(payload): Json<MpesaCallbackRequest>,
) -> impl IntoResponse {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to start transaction: {}", e) }))
        ).into_response(),
    };

    // Calculate 30% commission
    let commission_rate = Decimal::from_str("30.00").unwrap();
    let commission_amount = payload.amount * (commission_rate / Decimal::from_str("100").unwrap());

    // Check if transaction already processed
    let existing = sqlx::query("SELECT id FROM commissions WHERE transaction_ref = $1")
        .bind(&payload.transaction_ref)
        .fetch_optional(&mut *tx)
        .await;

    if let Ok(Some(_)) = existing {
        let _ = tx.rollback().await;
        return (
            StatusCode::OK,
            Json(json!({ "message": "Transaction already processed" }))
        ).into_response();
    }

    // Update property subscription to active
    let prop_result = sqlx::query(
        r#"
        UPDATE properties
        SET subscription_status = 'active', subscription_expires_at = NOW() + INTERVAL '30 days'
        WHERE id = $1
        RETURNING id
        "#
    )
    .bind(payload.property_id)
    .fetch_one(&mut *tx)
    .await;

    if prop_result.is_err() {
        let _ = tx.rollback().await;
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Property not found" }))
        ).into_response();
    }

    // Credit agent's balance
    let agent_result = sqlx::query_as::<_, AccountUser>(
        r#"
        UPDATE account_users
        SET balance = COALESCE(balance, 0) + $1, updated_at = NOW()
        WHERE id = $2
        RETURNING *
        "#
    )
    .bind(commission_amount)
    .bind(payload.agent_id)
    .fetch_one(&mut *tx)
    .await;

    if agent_result.is_err() {
        let _ = tx.rollback().await;
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Agent not found" }))
        ).into_response();
    }

    // Log commission
    let commission_result = sqlx::query_as::<_, Commission>(
        r#"
        INSERT INTO commissions (agent_id, property_id, amount, commission_rate, transaction_ref, status)
        VALUES ($1, $2, $3, $4, $5, 'completed')
        RETURNING *
        "#
    )
    .bind(payload.agent_id)
    .bind(payload.property_id)
    .bind(commission_amount)
    .bind(commission_rate)
    .bind(&payload.transaction_ref)
    .fetch_one(&mut *tx)
    .await;

    match commission_result {
        Ok(commission) => {
            if let Err(e) = tx.commit().await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Failed to commit transaction: {}", e) }))
                ).into_response();
            }

            (StatusCode::OK, Json(json!({ 
                "message": "Commission processed successfully",
                "commission": commission
            }))).into_response()
        },
        Err(e) => {
            let _ = tx.rollback().await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to log commission: {}", e) }))
            ).into_response()
        }
    }
}

pub async fn list_commissions(
    State(pool): State<PgPool>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let agent_id = params.get("agent_id")
        .and_then(|s| Uuid::parse_str(s).ok());

    let commissions = if let Some(agent_id) = agent_id {
        sqlx::query_as::<_, Commission>(
            "SELECT * FROM commissions WHERE agent_id = $1 ORDER BY created_at DESC"
        )
        .bind(agent_id)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, Commission>(
            "SELECT * FROM commissions ORDER BY created_at DESC"
        )
        .fetch_all(&pool)
        .await
    };

    match commissions {
        Ok(commissions) => (StatusCode::OK, Json(json!({ "commissions": commissions }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to fetch commissions: {}", e) }))
        ).into_response(),
    }
}
