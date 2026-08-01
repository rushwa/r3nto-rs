// crates/api/src/handlers/choices.rs
use axum::Json;
use rento_core::error::Result;

pub async fn property_types() -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!([
        {"value": "single", "label": "Single Room"},
        {"value": "double", "label": "Double Room"},
        {"value": "bedsitter", "label": "Bedsitter"},
        {"value": "1bed", "label": "1 Bedroom"},
        {"value": "2bed", "label": "2 Bedrooms"},
        {"value": "3bed", "label": "3 Bedrooms"},
        {"value": "apartment", "label": "Apartment"},
        {"value": "bungalow", "label": "Bungalow"},
        {"value": "villa", "label": "Villa"},
        {"value": "land", "label": "Land"},
    ])))
}

pub async fn status_types() -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!([
        {"value": "vacant", "label": "Vacant"},
        {"value": "occupied", "label": "Occupied"},
        {"value": "to_let", "label": "To Let"},
        {"value": "available", "label": "Available"},
        {"value": "sold", "label": "Sold"},
        {"value": "under_offer", "label": "Under Offer"},
    ])))
}

pub async fn purpose_types() -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!([
        {"value": "rent", "label": "For Rent"},
        {"value": "sale", "label": "For Sale"},
    ])))
}
