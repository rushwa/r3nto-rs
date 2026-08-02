use axum::{
    extract::{Json, State, Multipart},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::middleware::auth::{RequireAuth, RequireAgentOrAdmin};
use rento_core::models::property::Property;

pub async fn upload_property_video(
    auth: RequireAgentOrAdmin,
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut property_id: Option<Uuid> = None;
    let mut latitude: Option<Decimal> = None;
    let mut longitude: Option<Decimal> = None;
    let mut video_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "property_id" => {
                if let Ok(text) = field.text().await {
                    property_id = Uuid::parse_str(&text).ok();
                }
            }
            "latitude" => {
                if let Ok(text) = field.text().await {
                    latitude = text.parse::<Decimal>().ok();
                }
            }
            "longitude" => {
                if let Ok(text) = field.text().await {
                    longitude = text.parse::<Decimal>().ok();
                }
            }
            "video" => {
                if let Ok(data) = field.bytes().await {
                    video_data = Some(data.to_vec());
                }
            }
            _ => {}
        }
    }

    let property_id = match property_id {
        Some(id) => id,
        None => return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing property_id" }))
        ).into_response(),
    };

    let latitude = match latitude {
        Some(lat) => lat,
        None => return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing latitude" }))
        ).into_response(),
    };

    let longitude = match longitude {
        Some(lon) => lon,
        None => return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing longitude" }))
        ).into_response(),
    };

    // TODO: Upload video to S3/Cloudinary and get URL
    let video_url = "https://example.com/video.mp4"; // Placeholder

    // Update property with verification data
    let result = sqlx::query_as::<_, Property>(
        r#"
        UPDATE properties
        SET video_url = $1, latitude = $2, longitude = $3, verified_at = NOW()
        WHERE id = $4
        RETURNING *
        "#
    )
    .bind(video_url)
    .bind(latitude)
    .bind(longitude)
    .bind(property_id)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(property) => (StatusCode::OK, Json(json!({ 
            "message": "Property verified successfully",
            "property": property
        }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to verify property: {}", e) }))
        ).into_response(),
    }
}

pub async fn get_viewing_url(
    _auth: RequireAuth,
    State(pool): State<PgPool>,
    axum::extract::Path(property_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    // Check if property has been viewed before
    let property = sqlx::query_as::<_, Property>(
        "SELECT * FROM properties WHERE id = $1"
    )
    .bind(property_id)
    .fetch_one(&pool)
    .await;

    let property = match property {
        Ok(p) => p,
        Err(_) => return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Property not found" }))
        ).into_response(),
    };

    let first_viewed_at = match property.first_viewed_at {
        Some(time) => time,
        None => {
            // First view - start the 2-hour countdown
            let now = Utc::now();
            let _ = sqlx::query("UPDATE properties SET first_viewed_at = $1 WHERE id = $2")
                .bind(now)
                .bind(property_id)
                .execute(&pool)
                .await;
            now
        }
    };

    // Check if 2 hours have passed
    let elapsed = Utc::now() - first_viewed_at;
    if elapsed.num_hours() >= 2 {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Viewing period has expired" }))
        ).into_response();
    }

    // Generate presigned URL (expires in remaining time)
    let remaining_seconds = (2 * 3600) - elapsed.num_seconds() as i64;
    // TODO: Generate actual presigned URL from S3/Cloudinary
    let presigned_url = format!("https://example.com/video.mp4?expires={}", remaining_seconds);

    (StatusCode::OK, Json(json!({ 
        "url": presigned_url,
        "expires_in_seconds": remaining_seconds
    }))).into_response()
}
