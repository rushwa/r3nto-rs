use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::context::admin_auth::AdminUser;

const API_BASE: &str = "http://localhost:8000";

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: AdminUser,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
    pub is_admin: bool,
    pub is_superuser: bool,
    pub is_staff: bool,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct GrantPrivilegesRequest {
    pub user_id: String,
    pub role: String,
    pub is_superuser: bool,
    pub is_staff: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: String,
    pub verified: bool,
    pub property_count: u32,
    pub commission_rate: f64,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Property {
    pub id: String,
    pub title: String,
    pub price: f64,
    pub status: String,
    pub owner: String,
    pub location: String,
    pub property_type: String,
    pub bedrooms: u32,
    pub bathrooms: u32,
    pub area_sqft: u32,
    pub created_at: String,
}

// #[derive(Deserialize, Clone, Debug)]
// pub struct PropertyDetail {
//     pub id: String,
//     pub title: String,
//     pub description: String,
//     pub price: f64,
//     pub status: String,
//     pub owner: User,
//     pub location: String,
//     pub property_type: String,
//     pub bedrooms: u32,
//     pub bathrooms: u32,
//     pub area_sqft: u32,
//     pub features: Vec<String>,
//     pub images: Vec<String>,
//     pub listing_date: String,
//     pub views: u32,
//     pub inquiries: u32,
// }

#[derive(Deserialize, Clone, Debug)]
pub struct StatsData {
    pub total_users: u32,
    pub total_agents: u32,
    pub total_properties: u32,
    pub total_revenue: f64,
    pub active_listings: u32,
    pub sold_this_month: u32,
    pub avg_price: f64,
    pub pending_commissions: u32,
    pub user_growth: String,
    pub revenue_growth: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ActivityItem {
    pub id: String,
    pub action: String,
    pub user: String,
    pub time: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub features: Vec<String>,
    pub subscribers: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Commission {
    pub id: String,
    pub agent: String,
    pub property: String,
    pub amount: f64,
    pub status: String,
    pub date: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Inquiry {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub property_id: String,
    pub property_title: String,
    pub message: String,
    pub status: String,
    pub created_at: String,
    pub assigned_to: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SalesData {
    pub month: String,
    pub sales: u32,
    pub revenue: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TopAgent {
    pub id: String,
    pub name: String,
    pub sales: u32,
    pub revenue: f64,
    pub commission: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MarketTrend {
    pub area: String,
    pub avg_price: f64,
    pub price_change: f64,
    pub volume: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemSettings {
    pub company_name: String,
    pub commission_rate: f64,
    pub maintenance_mode: bool,
    pub allow_registration: bool,
}

#[derive(Serialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub phone_number: Option<String>,
}

#[derive(Serialize)]
pub struct ToggleUserActiveRequest {
    pub user_id: String,
    pub is_active: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub phone_number: Option<String>,
    pub identification_no: Option<String>,
    pub county: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub location: Option<String>,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub phone_verified: bool,
    pub subscribed: bool,
    pub date_joined: String,
    pub last_login: Option<String>,
}

pub async fn toggle_user_active(token: &str, req: &ToggleUserActiveRequest) -> Result<(), String> {
    let body = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let _: serde_json::Value = fetch_json(&format!("/admin/users/{}/toggle-active", req.user_id), "POST", Some(token), Some(body)).await?;
    Ok(())
}

pub async fn get_user_profile(token: &str, id: &str) -> Result<UserProfile, String> {
    fetch_json(&format!("/admin/users/{}", id), "GET", Some(token), None).await
}

pub async fn create_user(token: &str, req: &CreateUserRequest) -> Result<(), String> {
    let body = serde_json::to_string(req).map_err(|e| e.to_string())?;
    fetch_json("/admin/users", "POST", Some(token), Some(body)).await
}

pub async fn grant_admin_privileges(token: &str, req: &GrantPrivilegesRequest) -> Result<(), String> {
    let body = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let _: serde_json::Value = fetch_json("/admin/grant-privileges", "POST", Some(token), Some(body)).await?;
    Ok(())
}


async fn fetch_json<T: serde::de::DeserializeOwned>(
    path: &str,
    method: &str,
    token: Option<&str>,
    body: Option<String>,
) -> Result<T, String> {
    let mut opts = RequestInit::new();
    opts.method(method);
    opts.mode(RequestMode::Cors);

    if let Some(b) = body {
        opts.body(Some(&wasm_bindgen::JsValue::from_str(&b)));
    }

    let url = format!("{}{}", API_BASE, path);

    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    request
        .headers()
        .set("X-Admin-Origin", "rento-admin")
        .map_err(|e| format!("Header error: {:?}", e))?;

    if let Some(t) = token {
        request
            .headers()
            .set("Authorization", &format!("Bearer {}", t))
            .map_err(|e| format!("Auth header error: {:?}", e))?;
    }

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|e| format!("Response cast error: {:?}", e))?;

    if !resp.ok() {
        return Err(format!("HTTP {}: {}", resp.status(), resp.status_text()));
    }

    let text = JsFuture::from(
        resp.text()
            .map_err(|e| format!("Text error: {:?}", e))?
    )
        .await
        .map_err(|e| format!("Text read error: {:?}", e))?
        .as_string()
        .ok_or("Response not text")?;

    serde_json::from_str(&text).map_err(|e| format!("JSON error: {}", e))
}

pub async fn admin_login(email: String, password: String) -> Result<LoginResponse, String> {
    let body = serde_json::to_string(&LoginRequest { email, password })
        .map_err(|e| e.to_string())?;
    fetch_json("/admin/login", "POST", None, Some(body)).await
}

pub async fn get_current_admin(token: &str) -> Result<AdminUser, String> {
    fetch_json("/admin/me", "GET", Some(token), None).await
}

pub async fn get_stats(token: &str) -> Result<StatsData, String> {
    fetch_json("/admin/stats", "GET", Some(token), None).await
}

pub async fn get_users(token: &str) -> Result<Vec<User>, String> {
    fetch_json("/admin/users", "GET", Some(token), None).await
}

pub async fn get_agents(token: &str) -> Result<Vec<Agent>, String> {
    fetch_json("/admin/agents", "GET", Some(token), None).await
}

pub async fn get_properties(token: &str) -> Result<Vec<Property>, String> {
    fetch_json("/admin/properties", "GET", Some(token), None).await
}

// pub async fn get_property_detail(token: &str, id: &str) -> Result<PropertyDetail, String> {
//     fetch_json(&format!("/admin/properties/{}", id), "GET", Some(token), None).await
// }

pub async fn get_subscription_plans(token: &str) -> Result<Vec<SubscriptionPlan>, String> {
    fetch_json("/admin/subscriptions/plans", "GET", Some(token), None).await
}

pub async fn get_commissions(token: &str) -> Result<Vec<Commission>, String> {
    fetch_json("/admin/commissions", "GET", Some(token), None).await
}

pub async fn get_inquiries(token: &str) -> Result<Vec<Inquiry>, String> {
    fetch_json("/admin/inquiries", "GET", Some(token), None).await
}

pub async fn update_inquiry_status(token: &str, inquiry_id: &str, status: &str, assigned_to: Option<&str>) -> Result<(), String> {
    #[derive(Serialize)]
    struct UpdateBody {
        status: String,
        assigned_to: Option<String>,
    }
    let body = serde_json::to_string(&UpdateBody {
        status: status.to_string(),
        assigned_to: assigned_to.map(|s| s.to_string()),
    }).map_err(|e| e.to_string())?;
    fetch_json(&format!("/admin/inquiries/{}", inquiry_id), "POST", Some(token), Some(body)).await
}

pub async fn get_sales_data(token: &str) -> Result<Vec<SalesData>, String> {
    fetch_json("/admin/analytics/sales", "GET", Some(token), None).await
}

pub async fn get_top_agents(token: &str) -> Result<Vec<TopAgent>, String> {
    fetch_json("/admin/analytics/top-agents", "GET", Some(token), None).await
}

pub async fn get_market_trends(token: &str) -> Result<Vec<MarketTrend>, String> {
    fetch_json("/admin/analytics/market-trends", "GET", Some(token), None).await
}

pub async fn get_settings(token: &str) -> Result<SystemSettings, String> {
    fetch_json("/admin/settings", "GET", Some(token), None).await
}

pub async fn update_settings(token: &str, settings: &SystemSettings) -> Result<(), String> {
    let body = serde_json::to_string(settings).map_err(|e| e.to_string())?;
    fetch_json("/admin/settings", "POST", Some(token), Some(body)).await
}

#[derive(Serialize)]
pub struct CreatePropertyRequest {
    pub title: String,
    pub description: Option<String>,
    pub price: f64,
    pub property_type: String,
    pub location: String,
    pub county: Option<String>,
}

pub async fn create_property(token: &str, req: &CreatePropertyRequest) -> Result<String, String> {
    let body = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let resp: serde_json::Value = fetch_json("/admin/properties", "POST", Some(token), Some(body)).await?;
    Ok(resp.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string())
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct PropertyOwner {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct PropertyDetail {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub status: String,
    pub owner: PropertyOwner,
    pub location: String,
    pub property_type: String,
    pub bedrooms: u32,
    pub bathrooms: u32,
    pub area_sqft: u32,
    pub features: Vec<String>,
    pub images: Vec<String>,
    pub listing_date: String,
    pub views: u32,
    pub inquiries: u32,
}

pub async fn get_property_detail(token: &str, property_id: &str) -> Result<PropertyDetail, String> {
    fetch_json(&format!("/admin/properties/{}", property_id), "GET", Some(token), None).await
}

// ───────────────────────────────────────────
// Digital Handshake (Email + UUID flow)
// ───────────────────────────────────────────
pub async fn initiate_handshake(token: &str, target_user_id: &str, target_email: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Req {
        target_user_id: String,
        target_email: String,
    }

    let body = serde_json::to_string(&Req {
        target_user_id: target_user_id.to_string(),
        target_email: target_email.to_string(),
    })
        .map_err(|e| e.to_string())?;

    let _: serde_json::Value = fetch_json(
        "/admin/agents/handshake/initiate",
        "POST",
        Some(token),
        Some(body),
    ).await?;

    Ok(())
}

pub async fn verify_handshake(token: &str, target_user_id: &str, target_email: &str, otp_code: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Req {
        target_user_id: String,
        target_email: String,
        otp_code: String,
    }

    let body = serde_json::to_string(&Req {
        target_user_id: target_user_id.to_string(),
        target_email: target_email.to_string(),
        otp_code: otp_code.to_string(),
    })
        .map_err(|e| e.to_string())?;

    let _: serde_json::Value = fetch_json(
        "/admin/agents/handshake/verify",
        "POST",
        Some(token),
        Some(body),
    ).await?;

    Ok(())
}