use axum::response::IntoResponse;
use serde_json::json;
use crate::middleware::auth::AdminUser;

/// Example protected admin route.
/// Axum automatically runs the `AdminUser` extractor before this handler executes.
pub async fn admin_dashboard(
    admin: AdminUser, 
    // State(state): State<AppState>, // Uncomment if you need DB/Redis access
) -> impl IntoResponse {
    axum::Json(json!({
        "message": "Welcome to the admin dashboard",
        "admin_user_id": admin.user_id.to_string(),
        "role": format!("{:?}", admin.claims.role)
    }))
}
