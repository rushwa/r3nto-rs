use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,

    pub owner_name: String,
    pub location: String,
    pub property_type: String,
    pub purpose: String,

    pub is_land: bool,
    pub plot_size: Option<String>,
    pub plot_dimensions: Option<String>,
    pub land_price: Option<f64>,

    pub unit_count: i64,
    pub min_unit_price: Option<f64>,
    pub max_unit_price: Option<f64>,

    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyOwner {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDetail {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,

    pub owner: PropertyOwner,

    pub county: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub location: Option<String>,
    pub village: Option<String>,
    pub display_location: String,

    pub property_type: String,
    pub purpose: String,

    pub is_land: bool,
    pub plot_size: Option<String>,
    pub plot_dimensions: Option<String>,
    pub land_price: Option<f64>,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub map_address: Option<String>,

    pub unit_count: i64,
    pub min_unit_price: Option<f64>,
    pub max_unit_price: Option<f64>,

    pub images: Vec<String>,
    pub listing_date: String,
    pub views: u32,
    pub inquiries: u32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PropertyDbRow {
    pub id: sqlx::types::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,

    pub owner_name: String,
    pub location: String,
    pub property_type: String,
    pub purpose: String,

    pub is_land: bool,
    pub plot_size: Option<String>,
    pub plot_dimensions: Option<String>,
    pub land_price: Option<f64>,

    pub unit_count: i64,
    pub min_unit_price: Option<f64>,
    pub max_unit_price: Option<f64>,

    pub created_at: DateTime<Utc>,
}

impl From<PropertyDbRow> for Property {
    fn from(row: PropertyDbRow) -> Self {
        Self {
            id: row.id.to_string(),
            title: row.title,
            description: row.description,
            status: row.status,
            owner_name: row.owner_name,
            location: row.location,
            property_type: row.property_type,
            purpose: row.purpose,
            is_land: row.is_land,
            plot_size: row.plot_size,
            plot_dimensions: row.plot_dimensions,
            land_price: row.land_price,
            unit_count: row.unit_count,
            min_unit_price: row.min_unit_price,
            max_unit_price: row.max_unit_price,
            created_at: row.created_at.format("%Y-%m-%d").to_string(),
        }
    }
}