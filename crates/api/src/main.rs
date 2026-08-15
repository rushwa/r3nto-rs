// crates/api/src/main.rs

use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    middleware::from_fn_with_state,
};
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

mod cli;
mod errors;
mod handlers;
mod middleware;
mod middlewares;
mod models;
mod services;
mod state;

use state::AppState;
use crate::middlewares::admin_auth::admin_auth_middleware;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rento_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let db = rento_core::db::Database::new(&database_url).await?;
    db.migrate().await?;

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    let auth_service = Arc::new(rento_core::auth::AuthService::new(jwt_secret.clone()));
    let email_service = Arc::new(
        rento_core::email::EmailService::from_env()
            .expect("Failed to initialize EmailService from environment")
    );

    // Near the top, after creating email_service:
    let mpesa_client = Arc::new(
        services::mpesa::MpesaClient::from_env()
            .expect("Failed to initialize M-Pesa client from environment")
    );


    // AppState is Clone — no Arc wrapper needed
    let state = AppState {
        db,
        auth: auth_service,
        jwt_secret,
        email: email_service, // <-- Added
        mpesa: mpesa_client,  // <-- NEW
    };

    // CLI mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "manage.py" {
        match args[2].as_str() {
            "createsuperuser" => cli::create_superuser(state).await.map_err(|e| anyhow::anyhow!("{}", e))?,
            "migrate" => tracing::info!("Migrations run automatically on startup."),
            "shell" => tracing::info!("Interactive shell not implemented. Use psql or your DB client."),
            cmd => {
                tracing::error!("Unknown command: {}. Available: createsuperuser, migrate, shell", cmd);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Mandatory superuser check
    let superuser_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM account_users WHERE is_superuser = true OR role = 'ADMIN'"
    )
        .fetch_one(&state.db.pool)
        .await?;

    if superuser_count == 0 {
        tracing::error!("NO SUPERUSER FOUND!");
        tracing::error!("Run: cargo run --bin rento-api -- manage.py createsuperuser");
        std::process::exit(1);
    }

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse()?,
            "http://127.0.0.1:3000".parse()?,
            "http://localhost:3001".parse()?,
            "http://127.0.0.1:3001".parse()?,
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::HeaderName::from_static("x-requested-with"),
            axum::http::header::HeaderName::from_static("x-admin-origin"),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600));

    // Public routes (no auth)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh_token))
        .route("/auth/verify-email", post(handlers::auth::request_email_otp))
        .route("/auth/verify-email-code", post(handlers::auth::verify_email_code))
        .route("/auth/verify-phone", post(handlers::auth::verify_phone))
        .route("/auth/activate", post(handlers::auth::activate_account))
        .route("/auth/resend-activation", post(handlers::auth::resend_activation))
        .route("/auth/password-reset", post(handlers::auth::request_password_reset))
        .route("/auth/password-reset/confirm", post(handlers::auth::confirm_password_reset))
        .route("/auth/username-reset", post(handlers::auth::request_username_reset))
        .route("/auth/oauth/:provider", get(handlers::auth::oauth_login))
        .route("/auth/oauth/:provider/callback", get(handlers::auth::oauth_callback))
        .route("/api/mpesa/callback", post(handlers::mpesa::mpesa_callback))
        .route("/choices/property-types", get(handlers::choices::property_types))
        .route("/choices/status-types", get(handlers::choices::status_types))
        .route("/choices/purpose-types", get(handlers::choices::purpose_types));

    // Protected auth routes
    let protected_routes = Router::new()
        .route("/auth/me", get(handlers::auth::me))
        .route("/auth/logout", post(handlers::auth::logout))
    .route("/api/payments/registration-fee", post(handlers::payments::pay_registration_fee))
        .route("/api/payments/subscription", post(handlers::payments::pay_subscription));

    // Admin routes (with admin auth middleware)
    let admin_routes = Router::new()
        .route("/admin/login", post(handlers::admin::login))
        .route("/admin/users", post(handlers::admin::create_user))
        .route("/admin/users/:id", get(handlers::admin::get_user_profile))
        .route("/admin/users/:id/toggle-active", post(handlers::admin::toggle_user_active))
        .route("/admin/me", get(handlers::admin::get_current_admin))
        .route("/admin/stats", get(handlers::admin::get_stats))
        .route("/admin/users", get(handlers::admin::get_users))
        .route("/admin/mpesa/stk-push", post(handlers::mpesa::initiate_payment))
        .route("/admin/agents", get(handlers::admin::get_agents))
        // NOTE: Ensure handlers::admin::get_properties accepts Extension(claims): Extension<Claims>
        .route("/admin/properties", get(handlers::admin::get_properties).post(handlers::admin::create_property))
        .route("/admin/properties/:id", get(handlers::admin::get_property_detail))
        .route("/admin/property-owners", get(handlers::admin::get_property_owners_with_status))
        // NEW: Route for agent leads
        .route("/admin/leads", get(handlers::admin::get_agent_leads))
        .route("/admin/agents/handshake/initiate", post(handlers::admin::initiate_handshake))
        .route("/admin/agents/handshake/verify", post(handlers::admin::verify_handshake))
        .route("/admin/agent-leads", get(handlers::admin::get_agent_leads))
        .route("/admin/agent-leads/:id/stage", post(handlers::admin::update_lead_stage))
        // Agent performance
        .route("/admin/agents/performance", get(handlers::admin::get_agent_performance))
        // Agent referrals
        .route("/admin/agents/referrals", get(handlers::admin::get_agent_referrals))
        .route("/admin/agents/referrals/record", post(handlers::admin::record_referral))
        // Bonus Tiers
        .route("/admin/agents/bonus-tiers", get(handlers::admin::get_bonus_tiers))
        .route("/admin/agents/bonus-progress", get(handlers::admin::get_my_bonus_progress))
        .route("/admin/agents/bonus-claim", post(handlers::admin::claim_bonus))
        // Leaderboard
        .route("/admin/agents/leaderboard", get(handlers::admin::get_leaderboard))
        // Virtual Tour System
        .route("/api/tours/request", post(handlers::admin::request_virtual_tour))
        .route("/api/tours/:id/confirm-payment", post(handlers::admin::confirm_tour_payment))
        .route("/api/tours/upload-video", post(handlers::admin::upload_tour_video))
        .route("/api/tours/:id/viewing-link", post(handlers::admin::generate_viewing_link))
        .route("/api/tours/view/:token", post(handlers::admin::access_tour_video))
        .route("/admin/properties/:id/delist", post(handlers::admin::delist_property))
        .route("/admin/agents/pending-tours", get(handlers::admin::get_agent_pending_tours))
        // B2C Payouts
        .route("/admin/payouts/b2c-history", get(handlers::admin::get_b2c_history))
        .route("/admin/payouts/:id/b2c", post(handlers::admin::process_b2c_payout))
        .route("/admin/registration-fee/status", get(handlers::admin::get_registration_fee_status))
        .route("/admin/subscriptions/my", get(handlers::admin::get_my_subscriptions))
        .route("/admin/subscriptions/plans", get(handlers::admin::get_subscription_plans))
        .route("/admin/payments/history", get(handlers::admin::get_payment_history))
        // Payout routes (agent + admin)
        .route("/admin/payouts/request", post(handlers::admin::request_payout))
        .route("/admin/payouts/my-history", get(handlers::admin::get_my_payout_history))
        .route("/admin/payouts", get(handlers::admin::get_all_payout_history))
        .route("/admin/payouts/stats", get(handlers::admin::get_payout_stats))
        .route("/admin/payouts/approve", post(handlers::admin::approve_payout))
        .route("/admin/payouts/reject", post(handlers::admin::reject_payout))
        .route("/admin/payments/summary", get(handlers::admin::get_payment_summary))
        .route("/admin/commissions/my/summary", get(handlers::admin::get_my_commissions_summary))
        .route("/admin/subscriptions/overview", get(handlers::admin::get_subscriptions_overview))
        .route("/admin/subscriptions/subscribe", post(handlers::admin::subscribe_property))
        .route("/admin/commissions", get(handlers::admin::get_commissions))
        .route("/admin/commissions/my", get(handlers::mpesa::get_my_commissions))
        .route("/admin/commissions/my/wallet", get(handlers::mpesa::get_my_wallet))
        .route("/admin/commissions/my/payout", post(handlers::mpesa::request_payout))
        .route("/admin/inquiries", get(handlers::admin::get_inquiries))
        .route("/admin/inquiries/:id", post(handlers::admin::update_inquiry_status))
        .route("/admin/owner-inquiries", get(handlers::admin::get_owner_inquiries))
        .route("/admin/owner-inquiries/:id/status", post(handlers::admin::update_owner_inquiry_status))
        .route("/admin/analytics/sales", get(handlers::admin::get_sales_data))
        .route("/admin/analytics/top-agents", get(handlers::admin::get_top_agents))
        .route("/admin/analytics/market-trends", get(handlers::admin::get_market_trends))
        .route("/admin/settings", get(handlers::admin::get_settings))
        .route("/admin/settings", post(handlers::admin::update_settings))
        .route("/admin/grant-privileges", post(handlers::admin::grant_admin_privileges))
        .layer(from_fn_with_state(state.clone(), admin_auth_middleware))
        .with_state(state.clone());

    let app = public_routes
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}