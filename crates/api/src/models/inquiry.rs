use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inquiry {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub property_id: String,
    pub property_title: String,
    pub message: String,
    pub status: String,
    pub created_at: String,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInquiryRequest {
    pub status: String,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InquiryDbRow {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub property_id: sqlx::types::Uuid,
    pub property_title: String,
    pub message: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub assigned_to: Option<String>,
}

impl From<InquiryDbRow> for Inquiry {
    fn from(row: InquiryDbRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            email: row.email,
            phone: row.phone,
            property_id: row.property_id.to_string(),
            property_title: row.property_title,
            message: row.message,
            status: row.status,
            created_at: row.created_at.format("%Y-%m-%d %H:%M").to_string(),
            assigned_to: row.assigned_to,
        }
    }
}
