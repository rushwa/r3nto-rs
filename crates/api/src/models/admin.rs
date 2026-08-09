use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

// JWT Claims — MUST match TokenClaims in core/models.rs
// ───────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,              // user ID as String
    pub email: String,
    pub role: String,             // ✅ String, not enum
    pub username: Option<String>, // ✅ Optional (admin tokens don't need it)
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: AdminUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatusResponse {
    pub superuser_exists: bool,
}

#[derive(Debug, Deserialize)]
pub struct ToggleUserActiveRequest {
    pub user_id: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub user_id: String,
    pub role: String,
    pub is_superuser: bool,
    pub is_staff: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GrantPrivilegesRequest {
    pub user_id: String,
    pub role: String,  // CLIENT | AGENT | PROPERTY_OWNER | ADMIN
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub phone_number: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub password_hash: Option<String>,
    pub status: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

impl From<UserRow> for AdminUser {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id.to_string(),
            email: row.email,
            name: row.name,
            role: row.role,
        }
    }
}
