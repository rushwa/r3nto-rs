use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: String,
    pub verified: bool,
    pub property_count: u32,
    pub commission_rate: f64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AgentDbRow {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub email: String,
    pub status: String,
    pub verified: bool,
    pub property_count: i64,
    pub commission_rate: f64,
}

impl From<AgentDbRow> for Agent {
    fn from(row: AgentDbRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            email: row.email,
            status: row.status,
            verified: row.verified,
            property_count: row.property_count as u32,
            commission_rate: row.commission_rate,
        }
    }
}