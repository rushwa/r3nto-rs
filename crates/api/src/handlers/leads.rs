use axum::{
    extract::{Json, State, Multipart},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::middleware::auth::{RequireAuth, RequireAgentOrAdmin};
use rento_core::models::agent::{AgentLead, ClaimLeadRequest, CreateLeadRequest, LeadStatus};

pub async fn create_lead(
    _auth: RequireAgentOrAdmin,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateLeadRequest>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, AgentLead>(
        r#"
        INSERT INTO agent_leads (email, full_name, phone, status)
        VALUES ($1, $2, $3, 'pending')
        RETURNING *
        "#
    )
    .bind(&payload.email)
    .bind(&payload.full_name)
    .bind(&payload.phone)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(lead) => (StatusCode::CREATED, Json(json!({ "lead": lead }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to create lead: {}", e) }))
        ).into_response(),
    }
}

pub async fn claim_lead(
    auth: RequireAgentOrAdmin,
    State(pool): State<PgPool>,
    Json(payload): Json<ClaimLeadRequest>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, AgentLead>(
        r#"
        UPDATE agent_leads
        SET claimed_by = $1, status = 'pending', updated_at = NOW()
        WHERE id = $2 AND claimed_by IS NULL
        RETURNING *
        "#
    )
    .bind(auth.0.user_id)
    .bind(payload.lead_id)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(lead) => (StatusCode::OK, Json(json!({ "lead": lead }))).into_response(),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Lead already claimed or not found" }))
        ).into_response(),
    }
}

pub async fn list_leads(
    _auth: RequireAgentOrAdmin,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let leads = sqlx::query_as::<_, AgentLead>(
        "SELECT * FROM agent_leads ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await;

    match leads {
        Ok(leads) => (StatusCode::OK, Json(json!({ "leads": leads }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to fetch leads: {}", e) }))
        ).into_response(),
    }
}
