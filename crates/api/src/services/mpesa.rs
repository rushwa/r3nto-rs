use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use rand::Rng;
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};
use crate::models::mpesa::{
    MpesaCallbackPayload, StkPushRequest, StkPushResponse,
};

// ───────────────────────────────────────────
// M-Pesa Client
// Currently in SIMULATION mode (no real Daraja calls).
// When you get Daraja credentials, uncomment the real methods below.
// ───────────────────────────────────────────

#[derive(Clone)]
pub struct MpesaClient {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub shortcode: String,
    pub passkey: String,
    pub callback_url: String,
    pub environment: String,
    // TODO: Uncomment when integrating real Daraja API
    // http: reqwest::Client,
    // access_token: Arc<RwLock<Option<CachedToken>>>,
}

// TODO: Uncomment when integrating real Daraja API
// #[derive(Clone)]
// struct CachedToken {
//     token: String,
//     expires_at: chrono::DateTime<Utc>,
// }

// TODO: Uncomment when integrating real Daraja API
// #[derive(serde::Deserialize)]
// struct OAuthResponse {
//     access_token: String,
//     expires_in: String,
// }

impl MpesaClient {
    pub fn from_env() -> ApiResult<Self> {
        Ok(Self {
            consumer_key: std::env::var("MPESA_CONSUMER_KEY")
                .unwrap_or_else(|_| "sandbox_key".into()),
            consumer_secret: std::env::var("MPESA_CONSUMER_SECRET")
                .unwrap_or_else(|_| "sandbox_secret".into()),
            shortcode: std::env::var("MPESA_SHORTCODE")
                .unwrap_or_else(|_| "174379".into()),
            passkey: std::env::var("MPESA_PASSKEY")
                .unwrap_or_else(|_| "sandbox_passkey".into()),
            callback_url: std::env::var("MPESA_CALLBACK_URL")
                .unwrap_or_else(|_| "http://localhost:8000/api/mpesa/callback".into()),
            environment: std::env::var("MPESA_ENVIRONMENT")
                .unwrap_or_else(|_| "sandbox".into()),
            // TODO: Uncomment when integrating real Daraja API
            // http: reqwest::Client::new(),
            // access_token: Arc::new(RwLock::new(None)),
        })
    }

    // ==========================================
    // SIMULATION MODE: Creates fake payment records
    // ==========================================
    pub async fn simulate_payment(
        &self,
        pool: &sqlx::PgPool,
        phone: &str,
        amount: u32,
        account_ref: &str,
    ) -> ApiResult<(String, String, String)> {
        let merchant_request_id = format!("SIM-{}", Uuid::new_v4());
        let checkout_request_id = format!(
            "ws_SIM_{}",
            Uuid::new_v4().to_string().replace("-", "").chars().take(16).collect::<String>()
        );
        let receipt_number = format!("SIM{}", rand::thread_rng().gen_range(100000..999999));

        sqlx::query(
            r#"
            INSERT INTO mpesa_transactions
                (merchant_request_id, checkout_request_id, mpesa_receipt_number,
                 phone_number, amount, transaction_type, status, result_code, result_desc)
            VALUES ($1, $2, $3, $4, $5, 'C2B', 'success', 0, 'Simulated successful payment')
            "#
        )
            .bind(&merchant_request_id)
            .bind(&checkout_request_id)
            .bind(&receipt_number)
            .bind(normalize_phone(phone)?)
            .bind(amount as f64)
            .execute(pool)
            .await?;

        tracing::info!(
            "💰 [SIMULATED] payment: receipt={}, amount=KES {}, phone={}, ref={}",
            receipt_number, amount, phone, account_ref
        );

        Ok((merchant_request_id, checkout_request_id, receipt_number))
    }

    // ==========================================
    // REAL DARAJA INTEGRATION (COMMENTED OUT)
    // Uncomment these when you have valid Daraja credentials
    // ==========================================

    /*
    fn base_url(&self) -> &str {
        if self.environment == "production" {
            "https://api.safaricom.co.ke"
        } else {
            "https://sandbox.safaricom.co.ke"
        }
    }

    pub async fn get_access_token(&self) -> ApiResult<String> {
        {
            let cache = self.access_token.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.expires_at > Utc::now() + chrono::Duration::minutes(5) {
                    return Ok(cached.token.clone());
                }
            }
        }

        let url = format!(
            "{}/oauth/v1/generate?grant_type=client_credentials",
            self.base_url()
        );

        let resp = self.http.get(&url)
            .basic_auth(&self.consumer_key, Some(&self.consumer_secret))
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("OAuth request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ApiError::Internal(format!("OAuth failed: {}", resp.status())));
        }

        let oauth: OAuthResponse = resp.json().await
            .map_err(|e| ApiError::Internal(format!("OAuth parse failed: {}", e)))?;

        let expires_in = oauth.expires_in.parse::<i64>().unwrap_or(2100);
        let cached = CachedToken {
            token: oauth.access_token.clone(),
            expires_at: Utc::now() + chrono::Duration::seconds(expires_in),
        };

        let mut cache = self.access_token.write().await;
        *cache = Some(cached);

        Ok(oauth.access_token)
    }

    fn generate_password(&self) -> (String, String) {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let raw = format!("{}{}{}", self.shortcode, self.passkey, timestamp);
        let password = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, raw.as_bytes());
        (password, timestamp)
    }

    pub async fn initiate_stk_push(
        &self,
        phone: &str,
        amount: u32,
        account_ref: &str,
        description: &str,
    ) -> ApiResult<StkPushResponse> {
        let token = self.get_access_token().await?;
        let (password, timestamp) = self.generate_password();
        let normalized_phone = normalize_phone(phone)?;

        let request = StkPushRequest {
            business_short_code: self.shortcode.clone(),
            password,
            timestamp,
            transaction_type: "CustomerPayBillOnline".into(),
            amount,
            party_a: normalized_phone.clone(),
            party_b: self.shortcode.clone(),
            phone_number: normalized_phone,
            callback_url: self.callback_url.clone(),
            account_reference: account_ref.chars().take(16).collect(),
            transaction_desc: description.chars().take(13).collect(),
        };

        let url = format!("{}/mpesa/stkpush/v1/processrequest", self.base_url());

        let resp = self.http.post(&url)
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("STK Push request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::Internal(format!("STK Push failed ({}): {}", status, body)));
        }

        let stk_resp: StkPushResponse = resp.json().await
            .map_err(|e| ApiError::Internal(format!("STK Push parse failed: {}", e)))?;

        if stk_resp.response_code != "0" {
            return Err(ApiError::Internal(format!("STK Push rejected: {}", stk_resp.response_description)));
        }

        Ok(stk_resp)
    }
    */
}

// ───────────────────────────────────────────
// Helpers
// ───────────────────────────────────────────

pub fn normalize_phone(phone: &str) -> ApiResult<String> {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    let normalized = if digits.starts_with("254") && digits.len() == 12 {
        digits
    } else if digits.starts_with("0") && digits.len() == 10 {
        format!("254{}", &digits[1..])
    } else if digits.starts_with("7") && digits.len() == 9 {
        format!("254{}", digits)
    } else {
        return Err(ApiError::BadRequest(
            format!("Invalid phone number: {}. Expected: 07XXXXXXXX or 2547XXXXXXXX", phone)
        ));
    };

    Ok(normalized)
}

pub fn extract_callback_value(
    metadata: &[crate::models::mpesa::CallbackMetadataItem],
    name: &str,
) -> Option<String> {
    metadata.iter()
        .find(|item| item.name == name)
        .and_then(|item| {
            item.value.as_str().map(|s| s.to_string())
                .or_else(|| item.value.as_i64().map(|n| n.to_string()))
        })
}