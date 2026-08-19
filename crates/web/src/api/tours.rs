// crates/web/src/api/tours.rs
use serde::{Deserialize, Serialize};
use crate::api::API_BASE;

#[derive(Debug, Serialize)]
pub struct TourRequestPayload {
    pub property_id: String,
    pub client_email: String,
    pub client_name: Option<String>,
    pub client_phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TourRequestResponse {
    pub request_id: String,
    pub fee_amount: f64,
    pub sla_deadline: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ConfirmPaymentPayload {
    pub payment_reference: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmPaymentResponse {
    pub message: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AccessTourPayload {
    pub device_fingerprint: String,
}

#[derive(Debug, Deserialize)]
pub struct AccessTourResponse {
    pub video_url: String,
    pub session_id: String,
    pub device_locked: bool,
    pub remaining_minutes: i64,
}

#[derive(Debug, Deserialize)]
pub struct ViewingLinkResponse {
    pub session_id: String,
    pub viewing_token: String,
    pub viewing_url: String,
    pub video_url: String,
    pub expires_at: String,
    pub window_minutes: i64,
}

pub async fn request_tour(payload: TourRequestPayload, token: &str) -> Result<TourRequestResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(&format!("{}/api/tours/request", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if resp.status().is_success() {
        resp.json::<TourRequestResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let err = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(err)
    }
}

pub async fn confirm_payment(request_id: &str, token: &str) -> Result<ConfirmPaymentResponse, String> {
    let client = reqwest::Client::new();
    let payload = ConfirmPaymentPayload {
        payment_reference: format!("SIM-MPESA-{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase()),
    };

    let resp = client
        .post(&format!("{}/api/tours/{}/confirm-payment", API_BASE, request_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if resp.status().is_success() {
        resp.json::<ConfirmPaymentResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let err = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(err)
    }
}

pub async fn access_tour(viewing_token: &str, device_fingerprint: &str) -> Result<AccessTourResponse, String> {
    let client = reqwest::Client::new();
    let payload = AccessTourPayload {
        device_fingerprint: device_fingerprint.to_string(),
    };

    let resp = client
        .post(&format!("{}/api/tours/view/{}", API_BASE, viewing_token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    if status.is_success() {
        resp.json::<AccessTourResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let err = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("{}: {}", status, err))
    }
}