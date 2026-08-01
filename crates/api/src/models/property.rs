use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub title: String,
    pub price: f64,
    pub status: String,
    pub owner: String,
    pub location: String,
    pub property_type: String,
    pub bedrooms: u32,
    pub bathrooms: u32,
    pub area_sqft: u32,
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
    pub description: String,
    pub price: f64,
    pub status: String,
    pub owner: PropertyOwner,
    pub location: String,
    pub property_type: String,
    pub bedrooms: u32,
    pub bathrooms: u32,
    pub area_sqft: u32,
    pub features: Vec<String>,
    pub images: Vec<String>,
    pub listing_date: String,
    pub views: u32,
    pub inquiries: u32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PropertyDbRow {
    pub id: sqlx::types::Uuid,
    pub title: String,
    pub price: f64,
    pub status: String,
    pub owner_name: String,
    pub location: String,
    pub property_type: String,
    pub bedrooms: i32,
    pub bathrooms: i32,
    pub area_sqft: i32,
    pub created_at: DateTime<Utc>,
}

impl From<PropertyDbRow> for Property {
    fn from(row: PropertyDbRow) -> Self {
        Self {
            id: row.id.to_string(),
            title: row.title,
            price: row.price,
            status: row.status,
            owner: row.owner_name,
            location: row.location,
            property_type: row.property_type,
            bedrooms: row.bedrooms as u32,
            bathrooms: row.bathrooms as u32,
            area_sqft: row.area_sqft as u32,
            created_at: row.created_at.format("%Y-%m-%d").to_string(),
        }
    }
}