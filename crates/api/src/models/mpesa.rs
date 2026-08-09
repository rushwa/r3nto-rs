use serde::{Deserialize, Serialize};

// ───────────────────────────────────────────
// Daraja API Request/Response Types
// (For future real M-Pesa integration)
// ───────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct StkPushRequest {
    pub business_short_code: String,
    pub password: String,
    pub timestamp: String,
    pub transaction_type: String,
    pub amount: u32,
    pub party_a: String,
    pub party_b: String,
    pub phone_number: String,
    pub callback_url: String,
    pub account_reference: String,
    pub transaction_desc: String,
}

#[derive(Debug, Deserialize)]
pub struct StkPushResponse {
    pub merchant_request_id: String,
    pub checkout_request_id: String,
    pub response_code: String,
    pub response_description: String,
    pub customer_message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MpesaCallbackPayload {
    pub body: CallbackBody,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CallbackBody {
    pub stk_callback: StkCallback,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct StkCallback {
    pub merchant_request_id: String,
    pub checkout_request_id: String,
    pub result_code: i32,
    pub result_desc: String,
    pub callback_metadata: Option<Vec<CallbackMetadataItem>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CallbackMetadataItem {
    pub name: String,
    pub value: serde_json::Value,
}

// ───────────────────────────────────────────
// Application DTOs
// ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiatePaymentRequest {
    pub phone_number: String,
    pub amount: u32,
    pub payment_type: String,
    pub reference_id: Option<String>,
    pub account_reference: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub merchant_request_id: String,
    pub checkout_request_id: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletInfo {
    pub agent_id: String,
    pub balance: f64,
    pub pending_balance: f64,
    pub total_earned: f64,
    pub total_withdrawn: f64,
    pub mpesa_phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRequest {
    pub amount: f64,
    pub mpesa_phone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommissionLedgerEntry {
    pub id: String,
    pub agent_id: String,
    pub payment_id: String,
    pub property_owner_id: String,
    pub property_id: Option<String>,
    pub commission_type: String,
    pub gross_amount: f64,
    pub commission_rate: f64,
    pub commission_amount: f64,
    pub status: String,
    pub credited_at: Option<String>,
    pub created_at: String,
}