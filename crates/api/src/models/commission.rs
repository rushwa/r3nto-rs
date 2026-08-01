use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commission {
    pub id: String,
    pub agent: String,
    pub property: String,
    pub amount: f64,
    pub status: String,
    pub date: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CommissionDbRow {
    pub id: sqlx::types::Uuid,
    pub agent_name: String,
    pub property_title: String,
    pub amount: f64,
    pub status: String,
    pub date: chrono::NaiveDate,
}

impl From<CommissionDbRow> for Commission {
    fn from(row: CommissionDbRow) -> Self {
        Self {
            id: row.id.to_string(),
            agent: row.agent_name,
            property: row.property_title,
            amount: row.amount,
            status: row.status,
            date: row.date.to_string(),
        }
    }
}