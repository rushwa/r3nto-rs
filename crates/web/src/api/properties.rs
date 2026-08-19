// crates/web/src/api/properties.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PropertyOwner {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PropertyDetail {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
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