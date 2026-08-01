// crates/api/src/middleware/auth.rs

use axum::{
    extract::Request,
    extract::State,
    middleware::Next,
    response::Response,
    http::StatusCode,
};

use rento_core::models::{UserRole, AccountUser};

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub role: UserRole,
    pub username: String,
    pub email: String,
    pub is_staff: bool,
    pub is_superuser: bool,
}

impl AuthenticatedUser {
    pub fn is_admin(&self) -> bool {
        self.is_superuser || self.role == UserRole::Admin
    }

    pub fn is_agent(&self) -> bool {
        self.role == UserRole::Agent
    }

    pub fn is_property_owner(&self) -> bool {
        self.role == UserRole::PropertyOwner
    }

    pub fn is_client(&self) -> bool {
        self.role == UserRole::Client
    }
}

pub async fn auth_middleware(
    State(state): State<crate::state::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .or_else(|| {
            request.headers()
                .get("Cookie")
                .and_then(|c| c.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';').find_map(|cookie| {
                        let mut parts = cookie.trim().splitn(2, '=');
                        let name = parts.next()?;
                        let value = parts.next()?;
                        if name == "access_token" {
                            Some(value)
                        } else {
                            None
                        }
                    })
                })
        });

    if let Some(token) = token {
        match state.auth.verify_token(token) {
            Ok(claims) => {
                let user_result = sqlx::query_as::<_, AccountUser>(
                    "SELECT * FROM account_users WHERE id = $1"
                )
                    .bind(claims.sub)
                    .fetch_optional(&state.db.pool)
                    .await;

                if let Ok(Some(user)) = user_result {
                    let auth_user = AuthenticatedUser {
                        user_id: user.id,
                        role: user.role,
                        username: user.username,
                        email: user.email,
                        is_staff: user.is_staff,
                        is_superuser: user.is_superuser,
                    };
                    request.extensions_mut().insert(auth_user);
                }
            }
            Err(_) => {}
        }
    }

    Ok(next.run(request).await)
}

// Extractor that requires authentication
pub struct RequireAuth(pub AuthenticatedUser);

#[axum::async_trait]
impl axum::extract::FromRequestParts<crate::state::AppState> for RequireAuth {
    type Rejection = rento_core::error::RentoError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts.extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| rento_core::error::RentoError::Auth("Authentication required".to_string()))?;

        Ok(Self(auth_user))
    }
}

// Extractor that requires admin
pub struct RequireAdmin(pub AuthenticatedUser);

#[axum::async_trait]
impl axum::extract::FromRequestParts<crate::state::AppState> for RequireAdmin {
    type Rejection = rento_core::error::RentoError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts.extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| rento_core::error::RentoError::Auth("Authentication required".to_string()))?;

        if !auth_user.is_admin() {
            return Err(rento_core::error::RentoError::Authorization("Admin access required".to_string()));
        }

        Ok(Self(auth_user))
    }
}

// Extractor that requires staff
pub struct RequireStaff(pub AuthenticatedUser);

#[axum::async_trait]
impl axum::extract::FromRequestParts<crate::state::AppState> for RequireStaff {
    type Rejection = rento_core::error::RentoError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts.extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| rento_core::error::RentoError::Auth("Authentication required".to_string()))?;

        if !auth_user.is_admin() && !auth_user.is_staff {
            return Err(rento_core::error::RentoError::Authorization("Staff access required".to_string()));
        }

        Ok(Self(auth_user))
    }
}

// Extractor that requires agent or admin
pub struct RequireAgentOrAdmin(pub AuthenticatedUser);

#[axum::async_trait]
impl axum::extract::FromRequestParts<crate::state::AppState> for RequireAgentOrAdmin {
    type Rejection = rento_core::error::RentoError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts.extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| rento_core::error::RentoError::Auth("Authentication required".to_string()))?;

        if !auth_user.is_admin() && !auth_user.is_agent() {
            return Err(rento_core::error::RentoError::Authorization("Agent or admin access required".to_string()));
        }

        Ok(Self(auth_user))
    }
}

// Extractor that requires property owner
pub struct RequirePropertyOwner(pub AuthenticatedUser);

#[axum::async_trait]
impl axum::extract::FromRequestParts<crate::state::AppState> for RequirePropertyOwner {
    type Rejection = rento_core::error::RentoError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts.extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| rento_core::error::RentoError::Auth("Authentication required".to_string()))?;

        if !auth_user.is_admin() && !auth_user.is_property_owner() {
            return Err(rento_core::error::RentoError::Authorization("Property owner access required".to_string()));
        }

        Ok(Self(auth_user))
    }
}