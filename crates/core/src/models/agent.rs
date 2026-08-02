use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentLead {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub status: LeadStatus,
    pub claimed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "lead_status", rename_all = "lowercase")]
pub enum LeadStatus {
    Pending,
    Converted,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub struct ClaimLeadRequest {
    pub lead_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeadRequest {
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailOtp {
    pub id: Uuid,
    pub user_id: Uuid,
    pub otp: String,
    pub purpose: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub user_id: Uuid,
    pub otp: String,
    pub referring_agent_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PropertyVerificationRequest {
    pub property_id: Uuid,
    pub latitude: Decimal,
    pub longitude: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Commission {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub property_id: Uuid,
    pub amount: Decimal,
    pub commission_rate: Decimal,
    pub transaction_ref: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MpesaCallbackRequest {
    pub transaction_ref: String,
    pub amount: Decimal,
    pub property_id: Uuid,
    pub agent_id: Uuid,
}
