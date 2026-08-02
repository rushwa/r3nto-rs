
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use rento_core::{
    error::RentoError,
    models::{
        AccountUser, UserRole, VerificationPurpose,
        EmailOtp, UserResponse,
    },
    email::EmailService,
};

use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

// ───────────────────────────────────────────
// DTOs
// ───────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub phone_number: Option<String>,
    pub password: String,
    pub password_confirm: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub user_role: Option<String>,
    pub verification_code: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyCodeRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub detail: String,
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirmRequest {
    pub email: String,
    pub code: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResendActivationRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ActivateAccountRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct UsernameResetRequest {
    pub email: String,
    pub new_username: String,
}

// ───────────────────────────────────────────
// Registration
// ───────────────────────────────────────────

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Response, RentoError> {
    if let Err(errors) = req.validate() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Validation error".to_string(),
                error: format!("{:?}", errors.field_errors()),
            }),
        ).into_response());
    }

    if req.password != req.password_confirm {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Passwords do not match".to_string(),
                error: "Password mismatch".to_string(),
            }),
        ).into_response());
    }

    let existing = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE email = $1"
    )
        .bind(&req.email)
        .fetch_optional(&state.db.pool)
        .await?;

    if existing.is_some() {
        return Ok((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                detail: "Email already registered".to_string(),
                error: "Email exists".to_string(),
            }),
        ).into_response());
    }

    let otp = sqlx::query_as::<_, EmailOtp>(
        "SELECT * FROM email_otps WHERE email = $1 AND code = $2 AND purpose = $3 AND is_used = false AND expires_at > NOW()"
    )
        .bind(&req.email)
        .bind(&req.verification_code)
        .bind(VerificationPurpose::Registration)
        .fetch_optional(&state.db.pool)
        .await?;

    if otp.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Invalid or expired verification code".to_string(),
                error: "Invalid OTP".to_string(),
            }),
        ).into_response());
    }

    sqlx::query("UPDATE email_otps SET is_used = true WHERE email = $1 AND code = $2")
        .bind(&req.email)
        .bind(&req.verification_code)
        .execute(&state.db.pool)
        .await?;

    let password_hash = state.auth.hash_password(&req.password)?;

    let role = match req.user_role.as_deref() {
        Some("agent") => UserRole::Agent,
        Some("property_owner") => UserRole::PropertyOwner,
        Some("admin") => UserRole::Admin,
        _ => UserRole::Client,
    };

    let first_name = req.first_name.clone().unwrap_or_default();
    let last_name = req.last_name.clone().unwrap_or_default();
    let email = req.email.clone();
    let phone_number = req.phone_number.clone();

    let user_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO account_users (
            id, email, username, password_hash, first_name, last_name,
            role, is_staff, is_active, is_superuser, phone_verified,
            date_joined, subscribed
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
    )
        .bind(user_id)
        .bind(&email)
        .bind(&email)
        .bind(&password_hash)
        .bind(&first_name)
        .bind(&last_name)
        .bind(&role)
        .bind(false)
        .bind(true)
        .bind(false)
        .bind(false)
        .bind(now)
        .bind(false)
        .execute(&state.db.pool)
        .await?;

    let (access_token, refresh_token) = state.auth.generate_tokens(
        user_id, &role, &email, &email,
    )?;

    let user = UserResponse {
        id: user_id,
        email: email.clone(),
        username: email,
        first_name,
        last_name,
        role,
        phone_number,
        identification_no: None,
        county: None,
        constituency: None,
        ward: None,
        location: None,
        phone_verified: false,
        subscribed: false,
        is_active: true,
    };

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user,
        }),
    ).into_response())
}

// ───────────────────────────────────────────
// Login
// ───────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, RentoError> {
    if let Err(errors) = req.validate() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Validation error".to_string(),
                error: format!("{:?}", errors.field_errors()),
            }),
        ).into_response());
    }

    let user = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE email = $1"
    )
        .bind(&req.email)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| RentoError::NotFound("User not found".to_string()))?;

    let valid = state.auth.verify_password(&req.password, &user.password_hash)?;

    if !valid {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                detail: "Invalid email or password".to_string(),
                error: "Authentication failed".to_string(),
            }),
        ).into_response());
    }

    if !user.is_active {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                detail: "Account is deactivated".to_string(),
                error: "Account inactive".to_string(),
            }),
        ).into_response());
    }

    let (access_token, refresh_token) = state.auth.generate_tokens(
        user.id, &user.role, &user.username, &user.email,
    )?;

    let user_response = UserResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        role: user.role,
        phone_number: user.phone_number,
        identification_no: user.identification_no,
        county: user.county,
        constituency: user.constituency,
        ward: user.ward,
        location: user.location,
        phone_verified: user.phone_verified,
        subscribed: user.subscribed,
        is_active: user.is_active,
    };

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: user_response,
        }),
    ).into_response())
}

// ───────────────────────────────────────────
// Request Email OTP
// ───────────────────────────────────────────

pub async fn request_email_otp(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<Response, RentoError> {
    let code = rento_core::auth::AuthService::generate_verification_code();
    let expires_at = Utc::now() + Duration::minutes(10);

    sqlx::query("DELETE FROM email_otps WHERE email = $1")
        .bind(&req.email)
        .execute(&state.db.pool)
        .await?;

    sqlx::query(
        "INSERT INTO email_otps (id, email, code, purpose, is_used, created_at, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
        .bind(uuid::Uuid::new_v4())
        .bind(&req.email)
        .bind(&code)
        .bind(VerificationPurpose::Registration)
        .bind(false)
        .bind(Utc::now())
        .bind(expires_at)
        .execute(&state.db.pool)
        .await?;

    let email_service = EmailService::from_env()?;
    if let Err(e) = email_service.send_verification_code(&req.email, None, &code).await {
        tracing::warn!("Failed to send verification email: {}. Code for {}: {}", e, req.email, code);
    } else {
        tracing::info!("Verification email sent to {}", req.email);
    }

    tracing::info!("Verification code for {}: {}", req.email, code);

    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Verification code sent".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// Verify Email Code
// ───────────────────────────────────────────

pub async fn verify_email_code(
    State(state): State<AppState>,
    Json(req): Json<VerifyCodeRequest>,
) -> Result<Response, RentoError> {
    let otp = sqlx::query_as::<_, EmailOtp>(
        "SELECT * FROM email_otps WHERE email = $1 AND code = $2 AND purpose = $3 AND is_used = false AND expires_at > NOW()"
    )
        .bind(&req.email)
        .bind(&req.code)
        .bind(VerificationPurpose::Registration)
        .fetch_optional(&state.db.pool)
        .await?;

    if otp.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Invalid or expired verification code".to_string(),
                error: "Invalid OTP".to_string(),
            }),
        ).into_response());
    }

    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Code verified".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// Verify Email (alias)
// ───────────────────────────────────────────

pub async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyCodeRequest>,
) -> Result<Response, RentoError> {
    verify_email_code(State(state), Json(req)).await
}

// ───────────────────────────────────────────
// Verify Phone (stub)
// ───────────────────────────────────────────

pub async fn verify_phone(
    State(_state): State<AppState>,
    Json(_req): Json<VerifyCodeRequest>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Phone verification not yet implemented".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// Refresh Token
// ───────────────────────────────────────────

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Response, RentoError> {
    let claims = state.auth.verify_token(&req.refresh_token)?;
    let (access_token, refresh_token) = state.auth.generate_tokens(
        claims.sub, &claims.role, &claims.username, &claims.email,
    )?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: UserResponse {
                id: uuid::Uuid::nil(),
                email: "".to_string(),
                username: "".to_string(),
                first_name: "".to_string(),
                last_name: "".to_string(),
                role: UserRole::Client,
                phone_number: None,
                identification_no: None,
                county: None,
                constituency: None,
                ward: None,
                location: None,
                phone_verified: false,
                subscribed: false,
                is_active: false,
            },
        }),
    ).into_response())
}

// ───────────────────────────────────────────
// Get Current User
// ───────────────────────────────────────────

pub async fn me(
    State(state): State<AppState>,
    auth_user: RequireAuth,
) -> Result<Response, RentoError> {
    let user = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE id = $1"
    )
        .bind(auth_user.0.user_id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| RentoError::NotFound("User not found".to_string()))?;

    let user_response = UserResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        role: user.role,
        phone_number: user.phone_number,
        identification_no: user.identification_no,
        county: user.county,
        constituency: user.constituency,
        ward: user.ward,
        location: user.location,
        phone_verified: user.phone_verified,
        subscribed: user.subscribed,
        is_active: user.is_active,
    };

    Ok((StatusCode::OK, Json(user_response)).into_response())
}

// ───────────────────────────────────────────
// Logout
// ───────────────────────────────────────────

pub async fn logout(
    _auth_user: RequireAuth,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Logged out successfully".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// Password Reset
// ───────────────────────────────────────────

pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetRequest>,
) -> Result<Response, RentoError> {
    let user = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE email = $1"
    )
        .bind(&req.email)
        .fetch_optional(&state.db.pool)
        .await?;

    if user.is_none() {
        return Ok((
            StatusCode::OK,
            Json(MessageResponse { message: "If the email exists, a reset code has been sent".to_string() }),
        ).into_response());
    }

    let code = rento_core::auth::AuthService::generate_verification_code();
    let expires_at = Utc::now() + Duration::minutes(30);

    sqlx::query("DELETE FROM email_otps WHERE email = $1")
        .bind(&req.email)
        .execute(&state.db.pool)
        .await?;

    sqlx::query(
        "INSERT INTO email_otps (id, email, code, purpose, is_used, created_at, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
        .bind(uuid::Uuid::new_v4())
        .bind(&req.email)
        .bind(&code)
        .bind(VerificationPurpose::PasswordReset)
        .bind(false)
        .bind(Utc::now())
        .bind(expires_at)
        .execute(&state.db.pool)
        .await?;

    let email_service = EmailService::from_env()?;
    if let Err(e) = email_service.send_password_reset(&req.email, &code).await {
        tracing::warn!("Failed to send password reset email: {}. Code for {}: {}", e, req.email, code);
    } else {
        tracing::info!("Password reset email sent to {}", req.email);
    }

    tracing::info!("Password reset code for {}: {}", req.email, code);

    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "If the email exists, a reset code has been sent".to_string() }),
    ).into_response())
}

pub async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetConfirmRequest>,
) -> Result<Response, RentoError> {
    let otp = sqlx::query_as::<_, EmailOtp>(
        "SELECT * FROM email_otps WHERE email = $1 AND code = $2 AND purpose = $3 AND is_used = false AND expires_at > NOW()"
    )
        .bind(&req.email)
        .bind(&req.code)
        .bind(VerificationPurpose::PasswordReset)
        .fetch_optional(&state.db.pool)
        .await?;

    if otp.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Invalid or expired reset code".to_string(),
                error: "Invalid OTP".to_string(),
            }),
        ).into_response());
    }

    let password_hash = state.auth.hash_password(&req.new_password)?;

    sqlx::query("UPDATE account_users SET password_hash = $1 WHERE email = $2")
        .bind(&password_hash)
        .bind(&req.email)
        .execute(&state.db.pool)
        .await?;

    sqlx::query("UPDATE email_otps SET is_used = true WHERE email = $1 AND code = $2")
        .bind(&req.email)
        .bind(&req.code)
        .execute(&state.db.pool)
        .await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Password reset successful".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// Resend Activation
// ───────────────────────────────────────────

pub async fn resend_activation(
    State(state): State<AppState>,
    Json(req): Json<ResendActivationRequest>,
) -> Result<Response, RentoError> {
    let user = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE email = $1"
    )
        .bind(&req.email)
        .fetch_optional(&state.db.pool)
        .await?;

    if user.is_none() {
        return Ok((
            StatusCode::OK,
            Json(MessageResponse { message: "If the email exists, an activation code has been sent".to_string() }),
        ).into_response());
    }

    let code = rento_core::auth::AuthService::generate_verification_code();
    let expires_at = Utc::now() + Duration::minutes(10);

    sqlx::query("DELETE FROM email_otps WHERE email = $1")
        .bind(&req.email)
        .execute(&state.db.pool)
        .await?;

    sqlx::query(
        "INSERT INTO email_otps (id, email, code, purpose, is_used, created_at, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
        .bind(uuid::Uuid::new_v4())
        .bind(&req.email)
        .bind(&code)
        .bind(VerificationPurpose::Registration)
        .bind(false)
        .bind(Utc::now())
        .bind(expires_at)
        .execute(&state.db.pool)
        .await?;

    let email_service = EmailService::from_env()?;
    if let Err(e) = email_service.send_verification_code(&req.email, None, &code).await {
        tracing::warn!("Failed to send activation email: {}. Code for {}: {}", e, req.email, code);
    } else {
        tracing::info!("Activation email sent to {}", req.email);
    }

    tracing::info!("Activation code for {}: {}", req.email, code);

    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "If the email exists, an activation code has been sent".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// Activate Account
// ───────────────────────────────────────────

pub async fn activate_account(
    State(state): State<AppState>,
    Json(req): Json<ActivateAccountRequest>,
) -> Result<Response, RentoError> {
    let otp = sqlx::query_as::<_, EmailOtp>(
        "SELECT * FROM email_otps WHERE email = $1 AND code = $2 AND purpose = $3 AND is_used = false AND expires_at > NOW()"
    )
        .bind(&req.email)
        .bind(&req.code)
        .bind(VerificationPurpose::Registration)
        .fetch_optional(&state.db.pool)
        .await?;

    if otp.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Invalid or expired activation code".to_string(),
                error: "Invalid OTP".to_string(),
            }),
        ).into_response());
    }

    sqlx::query("UPDATE account_users SET is_active = true WHERE email = $1")
        .bind(&req.email)
        .execute(&state.db.pool)
        .await?;

    sqlx::query("UPDATE email_otps SET is_used = true WHERE email = $1 AND code = $2")
        .bind(&req.email)
        .bind(&req.code)
        .execute(&state.db.pool)
        .await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Account activated successfully".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// Username Reset
// ───────────────────────────────────────────

pub async fn request_username_reset(
    State(_state): State<AppState>,
    Json(_req): Json<UsernameResetRequest>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Username reset not yet implemented".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// OAuth (stubs)
// ───────────────────────────────────────────

pub async fn oauth_login(
    Path(_provider): Path<String>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "OAuth login not yet implemented".to_string() }),
    ).into_response())
}

pub async fn oauth_callback(
    Path(_provider): Path<String>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "OAuth callback not yet implemented".to_string() }),
    ).into_response())
}

// ───────────────────────────────────────────
// Google OAuth (backward compatibility)
// ───────────────────────────────────────────

pub async fn google_oauth(
    State(_state): State<AppState>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Google OAuth not yet implemented".to_string() }),
    ).into_response())
}

pub async fn google_oauth_callback(
    State(_state): State<AppState>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Google OAuth callback not yet implemented".to_string() }),
    ).into_response())
}

pub async fn facebook_oauth(
    State(_state): State<AppState>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Facebook OAuth not yet implemented".to_string() }),
    ).into_response())
}

pub async fn facebook_oauth_callback(
    State(_state): State<AppState>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Facebook OAuth callback not yet implemented".to_string() }),
    ).into_response())
}

pub async fn apple_oauth(
    State(_state): State<AppState>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Apple OAuth not yet implemented".to_string() }),
    ).into_response())
}

pub async fn apple_oauth_callback(
    State(_state): State<AppState>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, RentoError> {
    Ok((
        StatusCode::OK,
        Json(MessageResponse { message: "Apple OAuth callback not yet implemented".to_string() }),
    ).into_response())
}