use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsData {
    pub total_users: u32,
    pub total_agents: u32,
    pub total_properties: u32,
    pub total_revenue: f64,
    pub active_listings: u32,
    pub sold_this_month: u32,
    pub avg_price: f64,
    pub pending_commissions: u32,
    pub user_growth: String,
    pub revenue_growth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesData {
    pub month: String,
    pub sales: u32,
    pub revenue: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAgent {
    pub id: String,
    pub name: String,
    pub sales: u32,
    pub revenue: f64,
    pub commission: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTrend {
    pub area: String,
    pub avg_price: f64,
    pub price_change: f64,
    pub volume: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSettings {
    pub company_name: String,
    pub commission_rate: f64,
    pub maintenance_mode: bool,
    pub allow_registration: bool,
}
