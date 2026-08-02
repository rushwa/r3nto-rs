use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use rand::Rng;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::middleware::auth::{RequireAuth, RequireAgentOrAdmin};
use rento_core::models::agent::{EmailOtp, VerifyOtpRequest};
use rento_core::models::user::{AccountUser, UserRole};

pub async fn initiate_conversion(
    auth: RequireAgentOrAdmin,
    State(pool): State<PgPool>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = payload.get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid user_id" }))).into_response(),
    };

    // Generate 6-digit OTP
    let otp = format!("{:06}", rand::thread_rng().gen_range(0..1000000));
    let expires_at = Utc::now() + Duration::minutes(10);

    // Store OTP
    let result = sqlx::query_as::<_, EmailOtp>(
        r#"
        INSERT INTO email_otps (user_id, otp, purpose, expires_at)
        VALUES ($1, $2, 'role_conversion', $3)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(&otp)
    .bind(expires_at)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(_) => {
            // TODO: Send OTP via email
            (StatusCode::OK, Json(json!({ 
                "message": "OTP sent successfully",
                "otp": otp // Remove in production!
            }))).into_response()
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to generate OTP: {}", e) }))
        ).into_response(),
    }
}

pub async fn verify_conversion(
    auth: RequireAgentOrAdmin,
    State(pool): State<PgPool>,
    Json(payload): Json<VerifyOtpRequest>,
) -> impl IntoResponse {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to start transaction: {}", e) }))
        ).into_response(),
    };

    // Verify OTP
    let otp_record = sqlx::query_as::<_, EmailOtp>(
        r#"
        SELECT * FROM email_otps
        WHERE user_id = $1 AND otp = $2 AND purpose = 'role_conversion'
        AND expires_at > NOW() AND used_at IS NULL
        "#
    )
    .bind(payload.user_id)
    .bind(&payload.otp)
    .fetch_one(&mut *tx)
    .await;

    if otp_record.is_err() {
        let _ = tx.rollback().await;
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid or expired OTP" }))
        ).into_response();
    }

    // Mark OTP as used
    let _ = sqlx::query("UPDATE email_otps SET used_at = NOW() WHERE id = $1")
        .bind(otp_record.unwrap().id)
        .execute(&mut *tx)
        .await;

    // Upgrade user role to PROPERTY_OWNER
    let user_result = sqlx::query_as::<_, AccountUser>(
        r#"
        UPDATE account_users
        SET role = 'property_owner', referred_by = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING *
        "#
    )
    .bind(payload.referring_agent_id)
    .bind(payload.user_id)
    .fetch_one(&mut *tx)
    .await;

    match user_result {
        Ok(user) => {
            // Update lead status to converted
            let _ = sqlx::query(
                "UPDATE agent_leads SET status = 'converted', updated_at = NOW() WHERE email = $1"
            )
            .bind(&user.email)
            .execute(&mut *tx)
            .await;

            if let Err(e) = tx.commit().await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Failed to commit transaction: {}", e) }))
                ).into_response();
            }

            (StatusCode::OK, Json(json!({ 
                "message": "User successfully converted to Property Owner",
                "user": user
            }))).into_response()
        },
        Err(e) => {
            let _ = tx.rollback().await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to upgrade user: {}", e) }))
            ).into_response()
        }
    }
}
