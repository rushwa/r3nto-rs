use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use rento_core::models::property::Property;

#[derive(serde::Deserialize)]
pub struct PropertiesFeedQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn get_active_properties(
    State(pool): State<PgPool>,
    Query(params): Query<PropertiesFeedQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // High-performance query: only return properties with active, non-expired subscriptions
    let properties = sqlx::query_as::<_, Property>(
        r#"
        SELECT * FROM properties
        WHERE subscription_status = 'active'
        AND (subscription_expires_at IS NULL OR subscription_expires_at > NOW())
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&pool)
    .await;

    match properties {
        Ok(props) => {
            let total = sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM properties
                WHERE subscription_status = 'active'
                AND (subscription_expires_at IS NULL OR subscription_expires_at > NOW())
                "#
            )
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            (StatusCode::OK, Json(json!({ 
                "properties": props,
                "total": total,
                "page": page,
                "per_page": per_page
            }))).into_response()
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to fetch properties: {}", e) }))
        ).into_response(),
    }
}
