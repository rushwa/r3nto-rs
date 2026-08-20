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
    if path == "/admin/setup-status"
        || path == "/admin/login"
        || path.starts_with("/api/tours/view/")  // ✅ Public viewing (uses token auth)
        || path.starts_with("/api/tours/stream/")   // ✅ ADD: secure streaming (token-gated
        || path.starts_with("/uploads/")   // ✅ ADD THIS LINE — allow video streaming
    {
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
        || path.starts_with("/admin/agent-leads")      // ✅ NEW: Lead pipeline
        || path == "/admin/commissions"
        || path.starts_with("/admin/commissions/my")
        || path == "/admin/payouts/request"
        || path == "/admin/payouts/my-history"
        || path == "/admin/payouts/b2c-history"          // ✅ NEW
        || path.starts_with("/admin/agents/handshake/")
        || path == "/admin/agents/performance"            // ✅ NEW
        || path == "/admin/agents/referrals"              // ✅ NEW
        || path == "/admin/agents/referrals/record"      // ✅ NEW
        // ✅ NEW: Bonus tiers & leaderboard
        || path == "/admin/agents/bonus-tiers"
        || path == "/admin/agents/bonus-progress"
        || path == "/admin/agents/bonus-claim"
        || path == "/admin/agents/leaderboard"
        // ✅ NEW: Virtual tour endpoints
        || path == "/admin/agents/pending-tours"
        || path == "/api/tours/upload-video"
        || path == "/admin/agents/tour-history"
        || path == "/admin/agents/sla-stats"
        || (path.starts_with("/api/tours/") && path.ends_with("/viewing-link"));

    // ───────────────────────────────────────────
    // ✅ NEW: Routes any authenticated user can access (including CLIENT)
    // These are client-facing actions like requesting tours and paying
    // ───────────────────────────────────────────
    let is_client_allowed_route = path == "/api/tours/request"
        || path == "/api/tours/my-tours"   // ✅ ADD THIS
        || (path.starts_with("/api/tours/") && path.ends_with("/confirm-payment"))
        || (path.starts_with("/api/tours/") && path.ends_with("/viewing-link"));
    // ───────────────────────────────────────────
    // Property Owner allowed routes
    // ✅ FIX: Added all routes Property Owners need
    // ───────────────────────────────────────────
    let is_property_owner_allowed_route = path == "/admin/me"
        || path.starts_with("/admin/properties")           // View & create properties
        || path == "/admin/registration-fee/status"         // Check payment status
        || path.starts_with("/admin/subscriptions")         // View plans & subscribe
        || path == "/admin/commissions"                     // View commissions
        || path.starts_with("/admin/commissions/my")      // Wallet & payment history
        || path.starts_with("/admin/payments/")
        || path == "/admin/owner-inquiries"
        || path.starts_with("/admin/owner-inquiries/");
    // ───────────────────────────────────────────
    // Authorization Logic
    // ───────────────────────────────────────────
    if !is_admin_or_superuser
        && !(is_agent && is_agent_allowed_route)
        && !(is_property_owner && is_property_owner_allowed_route)
        && !is_client_allowed_route
    {
        return Err(StatusCode::FORBIDDEN);
    }

    // Extra security: Only superusers can grant privileges
    if path == "/admin/grant-privileges" && !is_admin_or_superuser {
        return Err(StatusCode::FORBIDDEN);
    }

    // Pass the claims to the handler
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}