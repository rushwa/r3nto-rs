//crates/api/src/middlewares/admin_auth.rs
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

    if claims.role != "admin" && claims.role != "superuser" {
        return Err(StatusCode::FORBIDDEN);
    }

    if path == "/admin/grant-privileges" && claims.role != "superuser" {
        return Err(StatusCode::FORBIDDEN);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}