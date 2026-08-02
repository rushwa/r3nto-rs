use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode, header::AUTHORIZATION},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use rento_core::models::UserRole;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID (UUID as string)
    pub role: UserRole,
    pub exp: usize,
    pub iat: usize,
}

// Helper struct to satisfy existing `auth_user.0.user_id` calls in your handlers
#[derive(Debug, Clone)]
pub struct AuthUserData {
    pub user_id: uuid::Uuid,
    pub role: UserRole,
}

pub enum AuthError {
    InvalidToken,
    MissingToken,
    InsufficientPermissions,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
        };
        (status, Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

fn extract_and_verify_claims(parts: &mut Parts) -> Result<Claims, AuthError> {
    let token = parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AuthError::MissingToken)?;

    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::HS256];

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");
    let key = DecodingKey::from_secret(secret.as_ref());
    
    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| AuthError::InvalidToken)?;

    Ok(token_data.claims)
}

// 1. RequireAuth (Tuple struct to match existing `auth_user.0.user_id` usage)
pub struct RequireAuth(pub AuthUserData);

impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
    {
        Box::pin(async move {
            let claims = extract_and_verify_claims(parts)?;
            let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;
            Ok(RequireAuth(AuthUserData { user_id, role: claims.role }))
        })
    }
}

// 2. AdminUser
pub struct AdminUser {
    pub user_id: uuid::Uuid,
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
    {
        Box::pin(async move {
            let claims = extract_and_verify_claims(parts)?;
            if claims.role != UserRole::Admin {
                return Err(AuthError::InsufficientPermissions);
            }
            let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;
            Ok(AdminUser { user_id, claims })
        })
    }
}

// 3. RequireStaff
pub struct RequireStaff(pub AuthUserData);

impl<S> FromRequestParts<S> for RequireStaff
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
    {
        Box::pin(async move {
            let claims = extract_and_verify_claims(parts)?;
            if claims.role != UserRole::Admin { // Adjust to UserRole::Staff if your enum has it
                return Err(AuthError::InsufficientPermissions);
            }
            let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;
            Ok(RequireStaff(AuthUserData { user_id, role: claims.role }))
        })
    }
}

// 4. RequireAgentOrAdmin
pub struct RequireAgentOrAdmin(pub AuthUserData);

impl<S> FromRequestParts<S> for RequireAgentOrAdmin
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
    {
        Box::pin(async move {
            let claims = extract_and_verify_claims(parts)?;
            if claims.role != UserRole::Agent && claims.role != UserRole::Admin {
                return Err(AuthError::InsufficientPermissions);
            }
            let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;
            Ok(RequireAgentOrAdmin(AuthUserData { user_id, role: claims.role }))
        })
    }
}
