use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub features: Vec<String>,
    pub subscribers: u32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SubscriptionPlanDbRow {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub price: f64,
    pub features: Vec<String>,
    pub subscribers: i64,
}

impl From<SubscriptionPlanDbRow> for SubscriptionPlan {
    fn from(row: SubscriptionPlanDbRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            price: row.price,
            features: row.features,
            subscribers: row.subscribers as u32,
        }
    }
}