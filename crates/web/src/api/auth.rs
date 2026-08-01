use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

const API_BASE: &str = "http://localhost:8000";

// ============== Types ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// Backend RegisterRequest fields:
// email, phone_number, password, password_confirm, first_name, last_name, user_role, verification_code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub phone_number: Option<String>,
    pub password: String,
    pub password_confirm: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub user_role: Option<String>,
    pub verification_code: String,
}

// Backend AuthResponse includes user directly!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

// Backend UserResponse - NO created_at/updated_at fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub phone_number: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub phone_verified: bool,
    pub subscribed: bool,
    pub identification_no: Option<String>,
    pub county: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpVerifyRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirmRequest {
    pub email: String,
    pub code: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

// ============== localStorage Helpers ==============

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = localStorage)]
    fn getItem(key: &str) -> Option<String>;

    #[wasm_bindgen(js_namespace = localStorage)]
    fn setItem(key: &str, value: &str);

    #[wasm_bindgen(js_namespace = localStorage)]
    fn removeItem(key: &str);
}

pub fn get_access_token() -> Option<String> {
    getItem("access_token")
}

pub fn get_refresh_token() -> Option<String> {
    getItem("refresh_token")
}

pub fn store_tokens(access: &str, refresh: &str) {
    setItem("access_token", access);
    setItem("refresh_token", refresh);
}

pub fn clear_tokens() {
    removeItem("access_token");
    removeItem("refresh_token");
}

// ============== HTTP Helpers ==============

async fn api_get<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let token = get_access_token();

    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(&format!("{}{}", API_BASE, path), &opts)
        .map_err(|e| format!("Request creation failed: {:?}", e))?;

    request.headers().set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t))
            .map_err(|e| format!("Auth header error: {:?}", e))?;
    }

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let resp: Response = resp_value.dyn_into().map_err(|e| format!("Response cast: {:?}", e))?;
    let status = resp.status();

    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON parse: {:?}", e))?;

    let text = js_sys::JSON::stringify(&json)
        .map_err(|e| format!("Stringify: {:?}", e))?
        .as_string()
        .ok_or("JSON stringify failed")?;

    if status >= 200 && status < 300 {
        serde_json::from_str(&text).map_err(|e| format!("Deserialize: {}", e))
    } else {
        let err: ApiError = serde_json::from_str(&text).unwrap_or(ApiError {
            error: "Unknown".to_string(),
            message: text.clone(),
        });
        Err(format!("{}: {}", err.error, err.message))
    }
}

async fn api_post<B: Serialize, T: for<'de> Deserialize<'de>>(path: &str, body: &B) -> Result<T, String> {
    let token = get_access_token();
    let body_json = serde_json::to_string(body).map_err(|e| e.to_string())?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&body_json));

    let request = Request::new_with_str_and_init(&format!("{}{}", API_BASE, path), &opts)
        .map_err(|e| format!("Request creation failed: {:?}", e))?;

    request.headers().set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t))
            .map_err(|e| format!("Auth header error: {:?}", e))?;
    }

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let resp: Response = resp_value.dyn_into().map_err(|e| format!("Response cast: {:?}", e))?;
    let status = resp.status();

    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON parse: {:?}", e))?;

    let text = js_sys::JSON::stringify(&json)
        .map_err(|e| format!("Stringify: {:?}", e))?
        .as_string()
        .ok_or("JSON stringify failed")?;

    if status >= 200 && status < 300 {
        serde_json::from_str(&text).map_err(|e| format!("Deserialize: {}", e))
    } else {
        let err: ApiError = serde_json::from_str(&text).unwrap_or(ApiError {
            error: "Unknown".to_string(),
            message: text.clone(),
        });
        Err(format!("{}: {}", err.error, err.message))
    }
}

// ============== API Functions ==============

pub async fn login(email: &str, password: &str) -> Result<AuthResponse, String> {
    let req = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };
    api_post("/auth/login", &req).await
}

pub async fn register(
    first_name: &str,
    last_name: &str,
    email: &str,
    phone: &str,
    password: &str,
    otp_code: &str,
) -> Result<AuthResponse, String> {
    let req = RegisterRequest {
        email: email.to_string(),
        phone_number: if phone.is_empty() { None } else { Some(phone.to_string()) },
        password: password.to_string(),
        password_confirm: password.to_string(),
        first_name: if first_name.is_empty() { None } else { Some(first_name.to_string()) },
        last_name: if last_name.is_empty() { None } else { Some(last_name.to_string()) },
        user_role: None,
        verification_code: otp_code.to_string(),
    };
    api_post("/auth/register", &req).await
}

pub async fn request_email_otp(email: &str) -> Result<(), String> {
    let req = OtpRequest {
        email: email.to_string(),
    };
    api_post::<_, serde_json::Value>("/auth/verify-email", &req).await?;
    Ok(())
}

pub async fn verify_email_code(email: &str, code: &str) -> Result<(), String> {
    let req = OtpVerifyRequest {
        email: email.to_string(),
        code: code.to_string(),
    };
    api_post::<_, serde_json::Value>("/auth/verify-email-code", &req).await?;
    Ok(())
}

pub async fn request_password_reset(email: &str) -> Result<(), String> {
    let req = PasswordResetRequest {
        email: email.to_string(),
    };
    api_post::<_, serde_json::Value>("/auth/password-reset", &req).await?;
    Ok(())
}

pub async fn confirm_password_reset(email: &str, code: &str, new_password: &str) -> Result<(), String> {
    let req = PasswordResetConfirmRequest {
        email: email.to_string(),
        code: code.to_string(),
        new_password: new_password.to_string(),
    };
    api_post::<_, serde_json::Value>("/auth/password-reset-confirm", &req).await?;
    Ok(())
}

pub async fn get_current_user() -> Result<UserResponse, String> {
    api_get("/auth/me").await
}

pub async fn logout_api() -> Result<(), String> {
    api_post::<_, serde_json::Value>("/auth/logout", &serde_json::json!({})).await?;
    Ok(())
}

pub async fn refresh_access_token() -> Result<AuthResponse, String> {
    let refresh = get_refresh_token().ok_or("No refresh token")?;
    let req = serde_json::json!({ "refresh_token": refresh });
    api_post("/auth/refresh", &req).await
}