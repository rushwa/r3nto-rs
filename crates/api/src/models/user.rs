use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserDbRow {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub is_admin: bool,
    pub is_superuser: bool,
    pub is_staff: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserDbRow> for User {
    fn from(row: UserDbRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            email: row.email,
            role: row.role,
            status: row.status,
            created_at: row.created_at.format("%Y-%m-%d").to_string(),
            is_admin: row.is_admin,
            is_superuser: row.is_superuser,
            is_staff: row.is_staff,
            is_active: row.is_active,
        }
    }
}