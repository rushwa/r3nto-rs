use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use crate::state::AppState;
use crate::models::admin::Claims;

pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    // Skip auth for public endpoints
    if path == "/admin/setup-status" || path == "/admin/login" {
        return Ok(next.run(request).await);
    }

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => token_data.claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    let role_upper = claims.role.to_uppercase();
    let is_admin_or_superuser = role_upper == "ADMIN" || role_upper == "SUPERUSER";
    let is_agent = role_upper == "AGENT";
    let is_property_owner = role_upper == "PROPERTY_OWNER";

    // ───────────────────────────────────────────
    // Agent allowed routes
    // ───────────────────────────────────────────
    let is_agent_allowed_route = path == "/admin/me"
        || path.starts_with("/admin/properties")
        || path == "/admin/leads"
        || path == "/admin/commissions"
        || path.starts_with("/admin/commissions/my")  // Agent wallet + commissions
        || path.starts_with("/admin/agents/handshake/");

    // ───────────────────────────────────────────
    // Property Owner allowed routes
    // ───────────────────────────────────────────
    let is_property_owner_allowed_route = path == "/admin/me"
        || path.starts_with("/admin/properties")
        || path == "/admin/commissions"
        || path.starts_with("/admin/commissions/my")  // Agent wallet + commissions
        || path == "/admin/registration-fee/status"   // ✅ NEW: Check payment status
        || path.starts_with("/api/payments/");        // ✅ NEW: Payment endpoints

    // Authorization Logic
    if !is_admin_or_superuser
        && !(is_agent && is_agent_allowed_route)
        && !(is_property_owner && is_property_owner_allowed_route)
    {
        return Err(StatusCode::FORBIDDEN);
    }

    // Extra security: Only superusers can grant privileges
    if path == "/admin/grant-privileges" && role_upper != "SUPERUSER" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Pass the claims to the handler
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}