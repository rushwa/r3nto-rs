// crates/api/src/handlers/users.rs

use axum::{
    extract::{State, Path},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use rento_core::{
    models::*,
    error::{RentoError, Result},
};
use crate::state::AppState;
use crate::middleware::auth::{RequireAuth, RequireStaff};

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    auth_user: RequireAuth,
) -> Result<Json<UserResponse>> {
    let user: AccountUser = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE id = $1"
    )
    .bind(auth_user.0.user_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(user.into()))
}

pub async fn update_me(
    State(state): State<Arc<AppState>>,
    auth_user: RequireAuth,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<UserResponse>> {
    let first_name = req.get("first_name").and_then(|v| v.as_str());
    let last_name = req.get("last_name").and_then(|v| v.as_str());
    let phone_number = req.get("phone_number").and_then(|v| v.as_str());

    sqlx::query(
        "UPDATE account_users SET first_name = COALESCE($1, first_name), last_name = COALESCE($2, last_name), phone_number = COALESCE($3, phone_number) WHERE id = $4"
    )
    .bind(first_name)
    .bind(last_name)
    .bind(phone_number)
    .bind(auth_user.0.user_id)
    .execute(&state.db.pool)
    .await?;

    let user: AccountUser = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE id = $1"
    )
    .bind(auth_user.0.user_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(user.into()))
}

pub async fn complete_profile(
    State(state): State<Arc<AppState>>,
    auth_user: RequireAuth,
    Json(req): Json<CompleteProfileRequest>,
) -> Result<Json<UserResponse>> {
    let user: AccountUser = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE id = $1"
    )
    .bind(auth_user.0.user_id)
    .fetch_one(&state.db.pool)
    .await?;

    if !user.is_active {
        return Err(RentoError::Authorization(
            "Please activate your account before completing your profile.".to_string()
        ));
    }

    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM account_users WHERE identification_no = $1 AND id != $2"
    )
    .bind(&req.identification_no)
    .bind(auth_user.0.user_id)
    .fetch_optional(&state.db.pool)
    .await?;

    if existing.is_some() {
        return Err(RentoError::Conflict("Identification number already in use".to_string()));
    }

    sqlx::query(
        "UPDATE account_users SET first_name = $1, last_name = $2, identification_no = $3, county = $4, constituency = $5, ward = $6, location = $7 WHERE id = $8"
    )
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&req.identification_no)
    .bind(&req.county)
    .bind(&req.constituency)
    .bind(&req.ward)
    .bind(&req.location)
    .bind(auth_user.0.user_id)
    .execute(&state.db.pool)
    .await?;

    let user: AccountUser = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE id = $1"
    )
    .bind(auth_user.0.user_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(user.into()))
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    _auth: RequireStaff,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<UserResponse>>> {
    let mut query = String::from("SELECT * FROM account_users WHERE 1=1");

    if let Some(role) = params.get("role") {
        query.push_str(&format!(" AND role = '{}'", role));
    }

    if let Some(search) = params.get("search") {
        query.push_str(&format!(
            " AND (email ILIKE '%{}%' OR first_name ILIKE '%{}%' OR last_name ILIKE '%{}%' OR username ILIKE '%{}%')",
            search, search, search, search
        ));
    }

    query.push_str(" ORDER BY date_joined DESC");

    let users: Vec<AccountUser> = sqlx::query_as::<_, AccountUser>(&query)
        .fetch_all(&state.db.pool)
        .await?;

    let responses: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();
    Ok(Json(responses))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    _auth: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>> {
    let user: AccountUser = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(user.into()))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConvertRoleRequest {
    pub new_role: String,
    pub identification_no: Option<String>,
    pub username: Option<String>,
}

pub async fn convert_role(
    State(state): State<Arc<AppState>>,
    auth_user: RequireAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<ConvertRoleRequest>,
) -> Result<Json<serde_json::Value>> {
    let user_to_convert: AccountUser = sqlx::query_as::<_, AccountUser>(
        "SELECT * FROM account_users WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    if user_to_convert.role != UserRole::Client {
        return Err(RentoError::BadRequest("Only clients can be converted to other roles".to_string()));
    }

    let new_role = match req.new_role.as_str() {
        "AGENT" => UserRole::Agent,
        "PROPERTY_OWNER" => UserRole::PropertyOwner,
        _ => return Err(RentoError::BadRequest("Invalid role".to_string())),
    };

    if new_role == UserRole::Agent {
        if !auth_user.0.is_admin() {
            return Err(RentoError::Authorization("Only administrators can create agents".to_string()));
        }
    } else {
        if !auth_user.0.is_admin() && !auth_user.0.is_agent() {
            return Err(RentoError::Authorization("Only administrators or agents can create property owners".to_string()));
        }

        if auth_user.0.is_agent() {
            if req.identification_no.is_none() || req.username.is_none() {
                return Err(RentoError::BadRequest("Identification number and username required".to_string()));
            }

            if user_to_convert.identification_no.as_ref() != req.identification_no.as_ref() || 
               user_to_convert.username != req.username.unwrap_or_default() {
                return Err(RentoError::BadRequest("Client verification failed".to_string()));
            }
        }
    }

    let old_role = user_to_convert.role.clone();

    sqlx::query("UPDATE account_users SET role = $1 WHERE id = $2")
        .bind(&new_role)
        .bind(id)
        .execute(&state.db.pool)
        .await?;

    if new_role == UserRole::Agent {
        sqlx::query(
            "INSERT INTO agent_profiles (id, user_id, agent_id, total_commissions, pending_commissions, paid_commissions, created_at, updated_at) VALUES ($1, $2, $3, 0, 0, 0, $4, $4)"
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(id)
        .bind(Utc::now())
        .execute(&state.db.pool)
        .await?;
    } else if new_role == UserRole::PropertyOwner {
        sqlx::query(
            "INSERT INTO property_owner_profiles (id, user_id, properties_owned, subscription_tier, created_at, updated_at) VALUES ($1, $2, 0, 'basic', $3, $3)"
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(Utc::now())
        .execute(&state.db.pool)
        .await?;

        if auth_user.0.is_agent() {
            let agent_profile: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM agent_profiles WHERE user_id = $1"
            )
            .bind(auth_user.0.user_id)
            .fetch_optional(&state.db.pool)
            .await?;

            if let Some((agent_profile_id,)) = agent_profile {
                sqlx::query(
                    "INSERT INTO commissions (id, agent_id, property_owner_id, amount, commission_percentage, status, created_at) VALUES ($1, $2, $3, 0, 10, 'PENDING', $4)"
                )
                .bind(Uuid::new_v4())
                .bind(agent_profile_id)
                .bind(id)
                .bind(Utc::now())
                .execute(&state.db.pool)
                .await?;

                sqlx::query(
                    "UPDATE agent_profiles SET pending_commissions = pending_commissions + 100 WHERE id = $1"
                )
                .bind(agent_profile_id)
                .execute(&state.db.pool)
                .await?;
            }
        }
    }

    Ok(Json(serde_json::json!({
        "detail": format!("User successfully converted from {} to {}.", old_role, new_role),
        "user_id": id,
        "old_role": old_role.to_string(),
        "new_role": new_role.to_string()
    })))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    auth_user: RequireStaff,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode> {
    if id == auth_user.0.user_id {
        return Err(RentoError::BadRequest("You cannot delete your own account".to_string()));
    }

    sqlx::query("DELETE FROM account_users WHERE id = $1")
        .bind(id)
        .execute(&state.db.pool)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    _auth: RequireStaff,
) -> Result<Json<serde_json::Value>> {
    let stats: (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE role = 'CLIENT') as total_clients,
            COUNT(*) FILTER (WHERE role = 'AGENT') as total_agents,
            COUNT(*) FILTER (WHERE role = 'PROPERTY_OWNER') as total_property_owners,
            COUNT(*) FILTER (WHERE role = 'ADMIN') as total_admins,
            COUNT(*) FILTER (WHERE subscribed = true) as subscribed_users
        FROM account_users
        WHERE email != 'AnonymousUser'
        "#
    )
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "total_users": stats.0,
        "total_clients": stats.1,
        "total_agents": stats.2,
        "total_property_owners": stats.3,
        "total_admins": stats.4,
        "subscribed_users": stats.5,
    })))
}
