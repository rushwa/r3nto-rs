use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use rand::Rng;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use std::path::PathBuf;
use tokio::fs;
use crate::errors::{ApiError, ApiResult};
use crate::models::admin::{AdminUser, Claims, CreateUserRequest, LoginRequest, LoginResponse, SetupStatusResponse};
use crate::models::user::{User, UserDbRow};
use crate::models::agent::{Agent, AgentDbRow};
use crate::models::property::{Property, PropertyDbRow, PropertyDetail, PropertyOwner};
use crate::models::subscription::{SubscriptionPlan, SubscriptionPlanDbRow};
use crate::models::commission::{Commission, CommissionDbRow};
use crate::models::inquiry::{Inquiry, InquiryDbRow};
use crate::models::analytics::{StatsData, SalesData, TopAgent, MarketTrend, SystemSettings};


fn pool(db: &rento_core::Database) -> &sqlx::PgPool {
    &db.pool
}

pub async fn check_superuser_exists(db: &rento_core::Database) -> ApiResult<SetupStatusResponse> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM account_users WHERE is_superuser = true OR role = 'ADMIN'"
    )
        .fetch_one(pool(db)).await?;
    Ok(SetupStatusResponse { superuser_exists: count > 0 })
}

pub async fn login(db: &rento_core::Database, jwt_secret: &str, req: LoginRequest) -> ApiResult<LoginResponse> {
    // 1. Fetch the actual 'role' column from the database
    let row: Option<(Uuid, String, String, String, String, String, bool, String)> = sqlx::query_as(
        "SELECT id, email, username, password_hash, first_name, last_name, is_superuser, role::text FROM account_users WHERE email = $1 AND (is_superuser = true OR role = 'ADMIN' OR is_staff = true)"
    )
        .bind(&req.email)
        .fetch_optional(pool(db)).await?;

    // 2. Destructure the new 'db_role' field
    let (user_id, email, username, password_hash, first_name, last_name, is_superuser, db_role) = match row {
        Some(r) => r,
        None => return Err(ApiError::Unauthorized("Invalid credentials".to_string())),
    };

    let parsed_hash = PasswordHash::new(&password_hash)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    let name = format!("{} {}", first_name, last_name).trim().to_string();

    // 3. Use the ACTUAL role from the DB, or SUPERUSER if they are a superuser
    let actual_role = db_role.to_uppercase();
    let role = if is_superuser {
        "SUPERUSER".to_string()
    } else {
        actual_role
    };

    let user = AdminUser {
        id: user_id.to_string(),
        email,
        name: if name.is_empty() { username } else { name },
        role: role, // Now correctly returns "AGENT", "PROPERTY_OWNER", "ADMIN", etc.
    };

    let token = generate_token_with_secret(&user, jwt_secret)?;
    Ok(LoginResponse { token, user })
}
pub async fn get_current_admin(db: &rento_core::Database, claims: &Claims) -> ApiResult<AdminUser> {
    // Fetch the ACTUAL role from the database, not hardcoded
    let row: (String, String, String, String, bool, String) = sqlx::query_as(
        "SELECT id::text, email, username, COALESCE(NULLIF(first_name || ' ' || last_name, ' '), username) as name, is_superuser, role::text FROM account_users WHERE id = $1"
    )
        .bind(Uuid::parse_str(&claims.sub).map_err(|e| ApiError::BadRequest(e.to_string()))?)
        .fetch_one(pool(db)).await?;

    let is_superuser = row.4;
    let actual_role = row.5.to_uppercase(); // Get real role: "AGENT", "PROPERTY_OWNER", "ADMIN"

    // Only override to SUPERUSER if they truly are a superuser
    let role = if is_superuser {
        "SUPERUSER".to_string()
    } else {
        actual_role
    };

    Ok(AdminUser {
        id: row.0,
        email: row.1,
        name: row.3.trim().to_string(),
        role: role, // Now returns the REAL role
    })
}
pub async fn get_stats(db: &rento_core::Database) -> ApiResult<StatsData> {
    let p = pool(db);
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM account_users").fetch_one(p).await?;
    let total_agents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM account_users WHERE role = 'AGENT'").fetch_one(p).await?;
    let total_properties: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM properties").fetch_one(p).await?;
    let active_listings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM properties WHERE status = 'available'").fetch_one(p).await?;
    let sold_this_month: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM properties WHERE status = 'occupied' AND EXTRACT(MONTH FROM updated_at) = EXTRACT(MONTH FROM NOW())"
    ).fetch_one(p).await?;
    let avg_price: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(price)::float8 FROM properties WHERE status = 'available'"
    ).fetch_one(p).await?;
    let total_revenue: Option<f64> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount)::float8, 0) FROM commissions WHERE status = 'PAID'"
    ).fetch_one(p).await?;
    let pending_commissions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM commissions WHERE status = 'PENDING'"
    ).fetch_one(p).await?;

    Ok(StatsData {
        total_users: total_users as u32,
        total_agents: total_agents as u32,
        total_properties: total_properties as u32,
        total_revenue: total_revenue.unwrap_or(0.0),
        active_listings: active_listings as u32,
        sold_this_month: sold_this_month as u32,
        avg_price: avg_price.unwrap_or(0.0),
        pending_commissions: pending_commissions as u32,
        user_growth: "+12%".to_string(),
        revenue_growth: "+8%".to_string(),
    })
}

pub async fn get_user_profile(db: &rento_core::Database, user_id: &str) -> ApiResult<serde_json::Value> {
    let id = Uuid::parse_str(user_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    let row = sqlx::query(
        r#"
        SELECT
            id, email, username, first_name, last_name, role::text,
            phone_number, identification_no, county, constituency, ward, location,
            is_active, is_staff, is_superuser, phone_verified, subscribed,
            date_joined, last_login
        FROM account_users
        WHERE id = $1
        "#
    )
        .bind(id)
        .fetch_one(pool(db)).await?;

    let profile = serde_json::json!({
        "id": row.try_get::<sqlx::types::Uuid, _>("id")?.to_string(),
        "email": row.try_get::<String, _>("email")?,
        "username": row.try_get::<String, _>("username")?,
        "first_name": row.try_get::<String, _>("first_name")?,
        "last_name": row.try_get::<String, _>("last_name")?,
        "role": row.try_get::<String, _>("role")?,
        "phone_number": row.try_get::<Option<String>, _>("phone_number")?,
        "identification_no": row.try_get::<Option<String>, _>("identification_no")?,
        "county": row.try_get::<Option<String>, _>("county")?,
        "constituency": row.try_get::<Option<String>, _>("constituency")?,
        "ward": row.try_get::<Option<String>, _>("ward")?,
        "location": row.try_get::<Option<String>, _>("location")?,
        "is_active": row.try_get::<bool, _>("is_active")?,
        "is_staff": row.try_get::<bool, _>("is_staff")?,
        "is_superuser": row.try_get::<bool, _>("is_superuser")?,
        "phone_verified": row.try_get::<bool, _>("phone_verified")?,
        "subscribed": row.try_get::<bool, _>("subscribed")?,
        "date_joined": row.try_get::<chrono::DateTime<chrono::Utc>, _>("date_joined")?.to_string(),
        "last_login": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_login")?.map(|d| d.to_string()),
    });

    Ok(profile)
}
pub async fn toggle_user_active(db: &rento_core::Database, user_id: &str, is_active: bool) -> ApiResult<()> {
    let id = Uuid::parse_str(user_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // Prevent disabling a superuser
    let is_superuser: bool = sqlx::query_scalar(
        "SELECT is_superuser FROM account_users WHERE id = $1"
    )
        .bind(id)
        .fetch_one(pool(db)).await?;

    if !is_active && is_superuser {
        return Err(ApiError::BadRequest("Cannot disable a superuser".to_string()));
    }

    sqlx::query("UPDATE account_users SET is_active = $1, updated_at = NOW() WHERE id = $2")
        .bind(is_active)
        .bind(id)
        .execute(pool(db)).await?;
    Ok(())
}

pub async fn get_users(db: &rento_core::Database) -> ApiResult<Vec<User>> {
    let rows: Vec<UserDbRow> = sqlx::query_as(
        r#"
        SELECT
            id,
            COALESCE(NULLIF(first_name || ' ' || last_name, ' '), username) as name,
            email,
            role::text,
            CASE WHEN is_active THEN 'active' ELSE 'inactive' END as status,
            (is_superuser OR role = 'ADMIN') as is_admin,
            is_superuser,
            is_staff,
            is_active,
            date_joined as created_at
        FROM account_users
        ORDER BY date_joined DESC
        "#
    )
        .fetch_all(pool(db)).await?;

    Ok(rows.into_iter().map(User::from).collect())
}
pub async fn get_agents(db: &rento_core::Database) -> ApiResult<Vec<Agent>> {
    let rows: Vec<AgentDbRow> = sqlx::query_as(
        r#"
        SELECT
            u.id,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as name,
            u.email,
            CASE WHEN u.is_active THEN 'active' ELSE 'inactive' END as status,
            u.phone_verified as verified,
            COUNT(p.id) as property_count,
            2.5::float8 as commission_rate
        FROM account_users u
        LEFT JOIN properties p ON p.owner_id = u.id
        WHERE u.role = 'AGENT'
        GROUP BY u.id, u.first_name, u.last_name, u.username, u.email, u.is_active, u.phone_verified
        ORDER BY u.username
        "#
    )
        .fetch_all(pool(db)).await?;

    Ok(rows.into_iter().map(Agent::from).collect())
}



// ───────────────────────────────────────────
// DIGITAL HANDSHAKE (UPDATED: requires email + UUID)
// ───────────────────────────────────────────

/// Agent initiates handshake by providing BOTH the client's UUID and email.
/// The system verifies they match in the database, then sends an OTP to the email.
// ───────────────────────────────────────────
// DIGITAL HANDSHAKE (Requires BOTH UUID and Email)
// ───────────────────────────────────────────

pub async fn initiate_handshake(
    db: &rento_core::Database,
    email_service: &rento_core::email::EmailService,
    agent_id: &str,
    target_user_id: &str,
    target_email: &str,
) -> ApiResult<()> {
    let agent_uuid = Uuid::parse_str(agent_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid Agent UUID: {}", e)))?;

    // 1. Parse target UUID (strictly requires a valid UUID format)
    let target_uuid = Uuid::parse_str(target_user_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid User ID format. Must be a valid UUID: {}", e)))?;

    // 2. Fetch both Agent and Target User details
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT
            COALESCE(NULLIF(a.first_name || ' ' || a.last_name, ' '), a.username) as agent_name,
            t.email as target_email,
            COALESCE(NULLIF(t.first_name || ' ' || t.last_name, ' '), t.username) as target_name,
            t.role::text as current_role
        FROM account_users a
        CROSS JOIN account_users t
        WHERE a.id = $1 AND t.id = $2
        "#
    )
        .bind(agent_uuid)
        .bind(target_uuid)
        .fetch_optional(pool(db))
        .await?;

    let (agent_name, found_email, target_name, current_role) = match row {
        Some(r) => r,
        None => return Err(ApiError::NotFound("Agent or Target user not found".to_string())),
    };

    // 3. SECURITY CHECK: Verify the provided email matches the UUID in the database
    if found_email.to_lowercase() != target_email.to_lowercase() {
        return Err(ApiError::BadRequest(
            "Email does not match the provided User ID. Please verify both with the client.".into()
        ));
    }

    // 4. Verify target is a CLIENT
    if current_role != "CLIENT" {
        return Err(ApiError::BadRequest(format!(
            "Target user is currently a '{}' and cannot be converted.",
            current_role
        )));
    }

    // 5. Generate OTP
    let otp = format!("{:06}", rand::thread_rng().gen_range(0..1000000));
    let expires_at = Utc::now() + Duration::minutes(15);

    // 6. Insert new OTP
    sqlx::query(
        "INSERT INTO email_otps (email, code, purpose, expires_at, is_used)
         VALUES ($1, $2, 'ROLE_CONVERSION', $3, false)
         ON CONFLICT (email) DO UPDATE
         SET code = $2, purpose = 'ROLE_CONVERSION', expires_at = $3, is_used = false"
    )
        .bind(&found_email)
        .bind(&otp)
        .bind(expires_at)
        .execute(pool(db))
        .await?;

    // 7. Send the email
    email_service
        .send_handshake_otp(&found_email, &target_name, &agent_name, &otp)
        .await
        .map_err(|e| {
            tracing::error!("Failed to send Handshake email: {}", e);
            ApiError::Internal(format!("Failed to send verification email: {}", e))
        })?;

    tracing::info!("🤝 Handshake OTP sent to {} by agent {}", found_email, agent_name);

    Ok(())
}

pub async fn verify_handshake(
    db: &rento_core::Database,
    agent_id: &str,
    target_user_id: &str,
    target_email: &str,
    otp_code: &str,
) -> ApiResult<()> {
    // 1. Parse UUIDs
    let agent_uuid = Uuid::parse_str(agent_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid Agent UUID: {}", e)))?;
    let target_uuid = Uuid::parse_str(target_user_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid User ID format: {}", e)))?;

    // 2. Look up user by UUID
    let user_check: Option<(String, String)> = sqlx::query_as(
        "SELECT email, role::text FROM account_users WHERE id = $1 AND is_active = true"
    )
        .bind(target_uuid)
        .fetch_optional(pool(db))
        .await?;

    let (found_email, _current_role) = match user_check {
        Some(u) => u,
        None => return Err(ApiError::NotFound("User not found with this User ID".into())),
    };

    // 3. SECURITY CHECK: Verify email matches
    if found_email.to_lowercase() != target_email.to_lowercase() {
        return Err(ApiError::BadRequest(
            "Email does not match the provided User ID.".into()
        ));
    }

    // 4. Verify OTP
    let otp_record: Option<(Uuid, bool, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, is_used, expires_at FROM email_otps
         WHERE email = $1 AND code = $2 AND purpose = 'ROLE_CONVERSION'
         ORDER BY created_at DESC LIMIT 1"
    )
        .bind(&found_email)
        .bind(otp_code)
        .fetch_optional(pool(db))
        .await?;

    let (otp_id, is_used, expires_at) = match otp_record {
        Some(record) => record,
        None => return Err(ApiError::Unauthorized("Invalid OTP code".into())),
    };

    if is_used {
        return Err(ApiError::BadRequest("This OTP has already been used".into()));
    }
    if chrono::Utc::now() > expires_at {
        return Err(ApiError::BadRequest("This OTP has expired. Please request a new one.".into()));
    }

    // 5. Mark OTP as used
    sqlx::query("UPDATE email_otps SET is_used = true WHERE id = $1")
        .bind(otp_id)
        .execute(pool(db))
        .await?;

    // 6. Promote to PROPERTY_OWNER + grant dashboard access
    sqlx::query(
        "UPDATE account_users SET role = 'PROPERTY_OWNER', is_staff = TRUE, updated_at = NOW() WHERE id = $1"
    )
        .bind(target_uuid)
        .execute(pool(db))
        .await?;

    // 7. Record the conversion relationship
    sqlx::query(
        "INSERT INTO agent_conversions (agent_id, property_owner_id, converted_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (property_owner_id) DO NOTHING"
    )
        .bind(agent_uuid)
        .bind(target_uuid)
        .execute(pool(db))
        .await?;

    tracing::info!(
        "✅ Digital Handshake complete: Agent {} converted {} (User ID: {}) to PROPERTY_OWNER",
        agent_id, found_email, target_uuid
    );

    Ok(())
}
pub async fn get_property_owners_with_status(db: &rento_core::Database) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            u.id::text,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as name,
            u.email,
            u.phone_number,
            CASE WHEN u.is_active THEN 'active' ELSE 'inactive' END as status,
            u.date_joined as created_at,
            EXISTS (
                SELECT 1 FROM payments p
                WHERE p.payer_id = u.id
                AND p.payment_type = 'registration_fee'
                AND p.status = 'completed'
            ) as has_paid_registration_fee,
            (SELECT COALESCE(NULLIF(a.first_name || ' ' || a.last_name, ' '), a.username)
             FROM agent_conversions ac
             JOIN account_users a ON ac.agent_id = a.id
             WHERE ac.property_owner_id = u.id
             LIMIT 1) as converted_by_agent,
            (SELECT COUNT(*) FROM properties pr WHERE pr.owner_id = u.id) as property_count
        FROM account_users u
        WHERE u.role = 'PROPERTY_OWNER'
        ORDER BY u.date_joined DESC
        "#
    )
        .fetch_all(pool(db))
        .await?;

    let owners: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "name": row.try_get::<String, _>("name").unwrap_or_default(),
            "email": row.try_get::<String, _>("email").unwrap_or_default(),
            "phone": row.try_get::<Option<String>, _>("phone_number").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "has_paid_registration_fee": row.try_get::<bool, _>("has_paid_registration_fee").unwrap_or(false),
            "converted_by_agent": row.try_get::<Option<String>, _>("converted_by_agent").unwrap_or_default(),
            "property_count": row.try_get::<i64, _>("property_count").unwrap_or(0),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
        })
    }).collect();

    Ok(owners)
}
// ───────────────────────────────────────────
// Check if a Property Owner has paid the registration fee
// ───────────────────────────────────────────
pub async fn has_paid_registration_fee(db: &rento_core::Database, user_id: &Uuid) -> ApiResult<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payments WHERE payer_id = $1 AND payment_type = 'registration_fee' AND status = 'completed'"
    )
        .bind(user_id)
        .fetch_one(pool(db))
        .await?;

    Ok(count > 0)
}
pub async fn get_properties(db: &rento_core::Database, claims: &Claims) -> ApiResult<Vec<Property>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    let role_upper = claims.role.to_uppercase();

    let rows: Vec<PropertyDbRow> = if role_upper == "AGENT" {
        // AGENT: Only see properties owned by property owners they converted
        sqlx::query_as(
            r#"
            SELECT
                p.id, p.title, COALESCE(p.price, 0)::float8 as price, p.status::text as status,
                COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as owner_name,
                COALESCE(p.county || ', ' || p.location, p.location, p.county, '') as location,
                COALESCE(p.property_type::text, '') as property_type,
                0 as bedrooms, 0 as bathrooms, 0 as area_sqft,
                p.created_at
            FROM properties p
            JOIN account_users u ON p.owner_id = u.id
            JOIN agent_conversions ac ON p.owner_id = ac.property_owner_id
            WHERE ac.agent_id = $1
            ORDER BY p.created_at DESC
            "#
        )
            .bind(user_id)
            .fetch_all(pool(db)).await?
    } else if role_upper == "PROPERTY_OWNER" {
        // PROPERTY_OWNER: Only see their own properties
        sqlx::query_as(
            r#"
            SELECT
                p.id, p.title, COALESCE(p.price, 0)::float8 as price, p.status::text as status,
                COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as owner_name,
                COALESCE(p.county || ', ' || p.location, p.location, p.county, '') as location,
                COALESCE(p.property_type::text, '') as property_type,
                0 as bedrooms, 0 as bathrooms, 0 as area_sqft,
                p.created_at
            FROM properties p
            JOIN account_users u ON p.owner_id = u.id
            WHERE p.owner_id = $1
            ORDER BY p.created_at DESC
            "#
        )
            .bind(user_id)
            .fetch_all(pool(db)).await?
    } else {
        // ADMIN/SUPERUSER: See all properties
        sqlx::query_as(
            r#"
            SELECT
                p.id, p.title, COALESCE(p.price, 0)::float8 as price, p.status::text as status,
                COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as owner_name,
                COALESCE(p.county || ', ' || p.location, p.location, p.county, '') as location,
                COALESCE(p.property_type::text, '') as property_type,
                0 as bedrooms, 0 as bathrooms, 0 as area_sqft,
                p.created_at
            FROM properties p
            JOIN account_users u ON p.owner_id = u.id
            ORDER BY p.created_at DESC
            "#
        )
            .fetch_all(pool(db)).await?
    };

    Ok(rows.into_iter().map(Property::from).collect())
}

pub async fn get_property_detail(db: &rento_core::Database, id: &str) -> ApiResult<PropertyDetail> {
    let property_id = Uuid::parse_str(id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    let row = sqlx::query(
        r#"
        SELECT
            p.id, p.title, p.description, COALESCE(p.price, 0)::float8 as price, p.status::text as status,
            COALESCE(p.county || ', ' || p.location, p.location, p.county, '') as location,
            COALESCE(p.property_type::text, '') as property_type,
            0 as bedrooms, 0 as bathrooms, 0 as area_sqft,
            '{}'::text[] as features, '{}'::text[] as images,
            p.created_at as listing_date,
            0 as views, 0 as inquiries,
            u.id as owner_id,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as owner_name,
            u.email as owner_email,
            u.role::text as owner_role
        FROM properties p
        JOIN account_users u ON p.owner_id = u.id
        WHERE p.id = $1
        "#
    )
        .bind(property_id)
        .fetch_one(pool(db)).await?;

    let owner = PropertyOwner {
        id: row.try_get::<sqlx::types::Uuid, _>("owner_id")?.to_string(),
        name: row.try_get("owner_name")?,
        email: row.try_get("owner_email")?,
        role: row.try_get("owner_role")?,
    };

    Ok(PropertyDetail {
        id: row.try_get::<sqlx::types::Uuid, _>("id")?.to_string(),
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        price: row.try_get::<f64, _>("price")?,
        status: row.try_get("status")?,
        owner,
        location: row.try_get("location")?,
        property_type: row.try_get("property_type")?,
        bedrooms: row.try_get::<i32, _>("bedrooms")? as u32,
        bathrooms: row.try_get::<i32, _>("bathrooms")? as u32,
        area_sqft: row.try_get::<i32, _>("area_sqft")? as u32,
        features: row.try_get::<Vec<String>, _>("features").unwrap_or_default(),
        images: row.try_get::<Vec<String>, _>("images").unwrap_or_default(),
        listing_date: row.try_get::<chrono::DateTime<chrono::Utc>, _>("listing_date")?.format("%Y-%m-%d").to_string(),
        views: row.try_get::<i32, _>("views")? as u32,
        inquiries: row.try_get::<i32, _>("inquiries")? as u32,
    })
}

pub async fn get_commissions(db: &rento_core::Database) -> ApiResult<Vec<Commission>> {
    let rows: Vec<CommissionDbRow> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name,
            p.title as property_title,
            c.amount::float8,
            c.status::text,
            COALESCE(c.paid_at::date, c.created_at::date) as date
        FROM commissions c
        JOIN account_users u ON c.agent_id = u.id
        JOIN properties p ON c.property_owner_id = p.owner_id
        ORDER BY c.created_at DESC
        LIMIT 100
        "#
    )
        .fetch_all(pool(db)).await?;

    Ok(rows.into_iter().map(Commission::from).collect())
}

pub async fn get_inquiries(db: &rento_core::Database) -> ApiResult<Vec<Inquiry>> {
    let rows: Vec<InquiryDbRow> = sqlx::query_as(
        r#"
        SELECT
            i.id, i.name, i.email, i.phone,
            i.property_id, p.title as property_title,
            i.message, i.status, i.created_at, i.assigned_to
        FROM admin_inquiries i
        JOIN properties p ON i.property_id = p.id
        ORDER BY i.created_at DESC
        "#
    )
        .fetch_all(pool(db)).await?;

    Ok(rows.into_iter().map(Inquiry::from).collect())
}

pub async fn update_inquiry_status(
    db: &rento_core::Database,
    inquiry_id: &str,
    status: &str,
    assigned_to: Option<&str>,
) -> ApiResult<()> {
    let id = Uuid::parse_str(inquiry_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    sqlx::query("UPDATE admin_inquiries SET status = $1, assigned_to = $2, updated_at = NOW() WHERE id = $3")
        .bind(status)
        .bind(assigned_to)
        .bind(id)
        .execute(pool(db)).await?;
    Ok(())
}

pub async fn get_sales_data(db: &rento_core::Database) -> ApiResult<Vec<SalesData>> {
    let rows = sqlx::query(
        r#"
        SELECT
            TO_CHAR(date_trunc('month', created_at), 'Mon') as month,
            COUNT(*) as sales,
            COALESCE(SUM(price)::float8, 0) as revenue
        FROM properties
        WHERE status = 'occupied' AND created_at >= NOW() - INTERVAL '6 months'
        GROUP BY date_trunc('month', created_at)
        ORDER BY date_trunc('month', created_at)
        "#
    )
        .fetch_all(pool(db)).await?;

    let mut sales_data = Vec::new();
    for row in rows {
        sales_data.push(SalesData {
            month: row.try_get("month")?,
            sales: row.try_get::<i64, _>("sales")? as u32,
            revenue: row.try_get::<f64, _>("revenue")?,
        });
    }
    Ok(sales_data)
}

pub async fn get_top_agents(db: &rento_core::Database) -> ApiResult<Vec<TopAgent>> {
    let rows = sqlx::query(
        r#"
        SELECT
            u.id,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as name,
            COUNT(p.id) as sales,
            COALESCE(SUM(p.price)::float8, 0) as revenue,
            COALESCE(SUM(c.amount)::float8, 0) as commission
        FROM account_users u
        LEFT JOIN properties p ON p.owner_id = u.id AND p.status = 'occupied'
        LEFT JOIN commissions c ON c.agent_id = u.id AND c.status = 'PAID'
        WHERE u.role = 'AGENT'
        GROUP BY u.id, u.first_name, u.last_name, u.username
        ORDER BY sales DESC
        LIMIT 10
        "#
    )
        .fetch_all(pool(db)).await?;

    let mut agents = Vec::new();
    for row in rows {
        agents.push(TopAgent {
            id: row.try_get::<sqlx::types::Uuid, _>("id")?.to_string(),
            name: row.try_get("name")?,
            sales: row.try_get::<i64, _>("sales")? as u32,
            revenue: row.try_get::<f64, _>("revenue")?,
            commission: row.try_get::<f64, _>("commission")?,
        });
    }
    Ok(agents)
}

pub async fn get_market_trends(db: &rento_core::Database) -> ApiResult<Vec<MarketTrend>> {
    let rows = sqlx::query(
        r#"
        SELECT
            COALESCE(county, location, 'Unknown') as area,
            COALESCE(AVG(price)::float8, 0) as avg_price,
            0.0::float8 as price_change,
            COUNT(*) as volume
        FROM properties
        WHERE status = 'available'
        GROUP BY county, location
        ORDER BY volume DESC
        LIMIT 10
        "#
    )
        .fetch_all(pool(db)).await?;

    let mut trends = Vec::new();
    for row in rows {
        trends.push(MarketTrend {
            area: row.try_get("area")?,
            avg_price: row.try_get::<f64, _>("avg_price")?,
            price_change: row.try_get::<f64, _>("price_change")?,
            volume: row.try_get::<i64, _>("volume")? as u32,
        });
    }
    Ok(trends)
}

pub async fn get_settings(db: &rento_core::Database) -> ApiResult<SystemSettings> {
    let row = sqlx::query(
        "SELECT company_name, commission_rate::float8, maintenance_mode, allow_registration FROM system_settings LIMIT 1"
    )
        .fetch_one(pool(db)).await?;

    Ok(SystemSettings {
        company_name: row.try_get("company_name")?,
        commission_rate: row.try_get::<f64, _>("commission_rate")?,
        maintenance_mode: row.try_get("maintenance_mode")?,
        allow_registration: row.try_get("allow_registration")?,
    })
}

pub async fn update_settings(db: &rento_core::Database, settings: &SystemSettings) -> ApiResult<()> {
    sqlx::query(
        "UPDATE system_settings SET company_name = $1, commission_rate = $2, maintenance_mode = $3, allow_registration = $4, updated_at = NOW() WHERE id = 1"
    )
        .bind(&settings.company_name)
        .bind(settings.commission_rate)
        .bind(settings.maintenance_mode)
        .bind(settings.allow_registration)
        .execute(pool(db)).await?;
    Ok(())
}

pub async fn grant_admin_privileges(db: &rento_core::Database, user_id: &str, grant: bool) -> ApiResult<()> {
    let id = Uuid::parse_str(user_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    let role = if grant { "ADMIN" } else { "CLIENT" };

    sqlx::query(
        "UPDATE account_users SET is_superuser = $1, is_staff = $1, role = $2, updated_at = NOW() WHERE id = $3"
    )
        .bind(grant)
        .bind(role)
        .bind(id)
        .execute(pool(db)).await?;
    Ok(())
}
// ───────────────────────────────────────────
// Full User Role Management
// ───────────────────────────────────────────
pub async fn update_user_role(
    db: &rento_core::Database,
    user_id: &Uuid,
    role: Option<&str>,
    is_staff: Option<bool>,
    is_superuser: Option<bool>,
) -> ApiResult<serde_json::Value> {
    // Fetch current user to know what to change
    let current: (String, bool, bool, bool) = sqlx::query_as(
        "SELECT role::text, is_staff, is_superuser, is_active FROM account_users WHERE id = $1"
    )
        .bind(user_id)
        .fetch_optional(pool(db))
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;

    let (current_role, current_is_staff, current_is_superuser, _is_active) = current;

    // Determine new values
    let new_role = role.unwrap_or(&current_role);
    let new_is_staff = is_staff.unwrap_or(current_is_staff);
    let new_is_superuser = is_superuser.unwrap_or(current_is_superuser);

    // Validate role
    let valid_roles = ["CLIENT", "AGENT", "PROPERTY_OWNER", "ADMIN", "SUPERUSER"];
    let role_upper = new_role.to_uppercase();
    if !valid_roles.contains(&role_upper.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid role '{}'. Must be one of: {}",
            new_role, valid_roles.join(", ")
        )));
    }

    // Prevent removing the last superuser
    if current_is_superuser && !new_is_superuser {
        let other_superusers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM account_users WHERE is_superuser = true AND id != $1"
        )
            .bind(user_id)
            .fetch_one(pool(db))
            .await?;

        if other_superusers == 0 {
            return Err(ApiError::BadRequest(
                "Cannot remove superuser status from the only superuser".into()
            ));
        }
    }

    // Update the user
    sqlx::query(
        "UPDATE account_users SET role = $1::text::user_role, is_staff = $2, is_superuser = $3, updated_at = NOW() WHERE id = $4"
    )
        .bind(&role_upper)
        .bind(new_is_staff)
        .bind(new_is_superuser)
        .bind(user_id)
        .execute(pool(db))
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to update user role: {}", e)))?;

    // If becoming an agent, create their wallet
    if role_upper == "AGENT" && current_role.to_uppercase() != "AGENT" {
        let _ = crate::services::wallet::get_or_create_wallet(pool(db), user_id).await;
        tracing::info!("💼 Created wallet for new agent {}", user_id);
    }

    tracing::info!(
        "🔐 User {} role updated: role={}, is_staff={}, is_superuser={}",
        user_id, role_upper, new_is_staff, new_is_superuser
    );

    Ok(serde_json::json!({
        "message": "User role updated successfully",
        "user_id": user_id.to_string(),
        "role": role_upper,
        "is_staff": new_is_staff,
        "is_superuser": new_is_superuser,
    }))
}
pub async fn create_user(db: &rento_core::Database, req: &CreateUserRequest) -> ApiResult<()> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let existing: Option<(String,)> = sqlx::query_as("SELECT email FROM account_users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(pool(db)).await?;

    if existing.is_some() {
        return Err(ApiError::BadRequest("Email already exists".to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .to_string();

    let user_id = Uuid::new_v4();
    let is_admin = req.role == "ADMIN";

    sqlx::query(
        r#"
        INSERT INTO account_users
            (id, email, username, password_hash, first_name, last_name, role,
             is_active, is_staff, is_superuser, phone_verified, date_joined, subscribed)
        VALUES ($1, $2, $3, $4, $5, $6, $7::text::user_role, TRUE, $8, $9, FALSE, NOW(), FALSE)
        "#
    )
        .bind(user_id)
        .bind(&req.email)
        .bind(&req.username)
        .bind(&password_hash)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.role)
        .bind(is_admin)   // is_staff
        .bind(is_admin)   // is_superuser
        .execute(pool(db)).await?;

    Ok(())
}
fn generate_token_with_secret(user: &AdminUser, secret: &str) -> ApiResult<String> {
    let now = Utc::now();
    let exp = (now + Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user.id.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
        username: Some(user.name.clone()), // ✅ Add username
        exp,
        iat,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
        .map_err(|e| ApiError::Internal(format!("Token generation failed: {}", e)))
}

// ───────────────────────────────────────────
// Payout Management
// ───────────────────────────────────────────

pub async fn get_pending_payouts(db: &rento_core::Database) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            pr.id::text, pr.amount::float8, pr.status, pr.mpesa_phone,
            pr.created_at, pr.processed_at,
            u.id::text as agent_id,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name,
            u.email as agent_email
        FROM payout_requests pr
        JOIN account_users u ON pr.agent_id = u.id
        ORDER BY pr.created_at DESC
        LIMIT 100
        "#
    )
        .fetch_all(pool(db))
        .await?;

    let payouts: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "amount": row.try_get::<f64, _>("amount").unwrap_or(0.0),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "mpesa_phone": row.try_get::<String, _>("mpesa_phone").unwrap_or_default(),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
            "processed_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("processed_at")
                .ok().flatten().map(|d| d.to_string()),
            "agent_id": row.try_get::<String, _>("agent_id").unwrap_or_default(),
            "agent_name": row.try_get::<String, _>("agent_name").unwrap_or_default(),
            "agent_email": row.try_get::<String, _>("agent_email").unwrap_or_default(),
        })
    }).collect();

    Ok(payouts)
}

pub async fn approve_payout(
    db: &rento_core::Database,
    email_service: &rento_core::email::EmailService,
    payout_id: &str,
) -> ApiResult<()> {
    let id = Uuid::parse_str(payout_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // Get payout details including agent info
    let payout_info: Option<(Uuid, Uuid, f64, String, String, String)> = sqlx::query_as(
        r#"
        SELECT pr.id, pr.agent_id, pr.amount::float8, pr.status, pr.mpesa_phone,
               COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name
        FROM payout_requests pr
        JOIN account_users u ON pr.agent_id = u.id
        WHERE pr.id = $1
        "#
    )
        .bind(id)
        .fetch_optional(pool(db))
        .await?;

    let (payout_uuid, agent_id, amount, status, phone, agent_name) = match payout_info {
        Some(p) => p,
        None => return Err(ApiError::NotFound("Payout not found".into())),
    };

    if status != "pending" {
        return Err(ApiError::BadRequest(format!("Payout is already {}", status)));
    }

    // Get agent email
    let agent_email: String = sqlx::query_scalar(
        "SELECT email FROM account_users WHERE id = $1"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    // Mark as approved and processed
    sqlx::query(
        "UPDATE payout_requests SET status = 'approved', processed_at = NOW() WHERE id = $1"
    )
        .bind(id)
        .execute(pool(db))
        .await?;

    // ✅ Send email notification
    if let Err(e) = email_service.send_payout_approved(&agent_email, &agent_name, amount, &phone).await {
        tracing::warn!("Failed to send payout approval email: {}", e);
        // Don't fail the whole operation if email fails
    }

    tracing::info!("✅ Payout {} approved for agent {} (KES {:.2})", payout_id, agent_name, amount);
    Ok(())
}

pub async fn reject_payout(
    db: &rento_core::Database,
    email_service: &rento_core::email::EmailService,
    payout_id: &str,
) -> ApiResult<()> {
    let id = Uuid::parse_str(payout_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // Get payout details including agent info
    let payout_info: Option<(Uuid, Uuid, f64, String, String)> = sqlx::query_as(
        r#"
        SELECT pr.id, pr.agent_id, pr.amount::float8, pr.status,
               COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name
        FROM payout_requests pr
        JOIN account_users u ON pr.agent_id = u.id
        WHERE pr.id = $1
        "#
    )
        .bind(id)
        .fetch_optional(pool(db))
        .await?;

    let (payout_uuid, agent_id, amount, status, agent_name) = match payout_info {
        Some(p) => p,
        None => return Err(ApiError::NotFound("Payout not found".into())),
    };

    if status != "pending" {
        return Err(ApiError::BadRequest(format!("Cannot reject payout with status: {}", status)));
    }

    // Get agent email
    let agent_email: String = sqlx::query_scalar(
        "SELECT email FROM account_users WHERE id = $1"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    let mut tx = pool(db).begin().await?;

    // Refund the wallet
    crate::services::wallet::credit_wallet(
        &mut tx,
        &agent_id,
        amount,
        &payout_uuid.to_string(),
        &format!("Payout {} rejected - funds refunded", payout_id),
    ).await?;

    // Mark as rejected
    sqlx::query(
        "UPDATE payout_requests SET status = 'rejected', processed_at = NOW() WHERE id = $1"
    )
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    // ✅ Send email notification
    if let Err(e) = email_service.send_payout_rejected(&agent_email, &agent_name, amount).await {
        tracing::warn!("Failed to send payout rejection email: {}", e);
    }

    tracing::info!("❌ Payout {} rejected, KES {:.2} refunded to agent {}", payout_id, amount, agent_name);
    Ok(())
}

pub async fn get_subscription_plans(db: &rento_core::Database) -> ApiResult<Vec<SubscriptionPlan>> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text,
            name,
            price::float8,
            CASE
                WHEN jsonb_typeof(features) = 'array' THEN
                    (SELECT array_agg(elem::text) FROM jsonb_array_elements_text(features) elem)
                ELSE '{}'::text[]
            END as features,
            0 as subscribers
        FROM subscription_plans
        WHERE is_active = true
        ORDER BY price
        "#
    )
        .fetch_all(pool(db))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch subscription plans: {}", e);
            ApiError::Internal(format!("Database error: {}", e))
        })?;

    let plans: Vec<SubscriptionPlan> = rows.into_iter().map(|row| {
        use sqlx::Row;
        SubscriptionPlan {
            id: row.try_get::<String, _>("id").unwrap_or_default(),
            name: row.try_get::<String, _>("name").unwrap_or_default(),
            price: row.try_get::<f64, _>("price").unwrap_or(0.0),
            features: row.try_get::<Vec<String>, _>("features").unwrap_or_default(),
            subscribers: row.try_get::<i64, _>("subscribers").unwrap_or(0) as u32,
        }
    }).collect();

    Ok(plans)
}
// ───────────────────────────────────────────
// Subscriptions Overview (per-property)
// ───────────────────────────────────────────
pub async fn get_subscriptions_overview(
    db: &rento_core::Database,
    user_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            p.id::text,
            p.title,
            p.price::float8,
            COALESCE(p.county || ', ' || p.location, p.location, p.county, 'Unknown') as location,
            p.status::text as property_status,
            p.subscription_status::text as current_sub_status,
            COALESCE(ps.plan_name, 'No Subscription') as plan_name,
            COALESCE(ps.plan_tier, 'none') as plan_tier,
            COALESCE(ps.plan_price::float8, 0.0) as plan_price,
            ps.start_date,
            ps.end_date,
            CASE
                WHEN ps.end_date IS NULL THEN 'none'
                WHEN ps.end_date < NOW() THEN 'expired'
                WHEN ps.end_date < NOW() + INTERVAL '7 days' THEN 'expiring'
                ELSE 'active'
            END as sub_status,
            CASE
                WHEN ps.end_date IS NULL THEN 0
                WHEN ps.end_date < NOW() THEN 0
                ELSE GREATEST(0, (ps.end_date::date - NOW()::date))
            END as days_remaining
        FROM properties p
        LEFT JOIN LATERAL (
            SELECT
                sp.name as plan_name,
                sp.tier::text as plan_tier,
                sp.price::float8 as plan_price,
                ps2.start_date,
                ps2.end_date
            FROM property_subscriptions ps2
            JOIN subscription_plans sp ON ps2.plan_id = sp.id
            WHERE ps2.property_id = p.id
              AND ps2.status = 'active'
            ORDER BY ps2.created_at DESC
            LIMIT 1
        ) ps ON true
        WHERE p.owner_id = $1
        ORDER BY
            CASE
                WHEN ps.end_date IS NULL THEN 3
                WHEN ps.end_date < NOW() THEN 2
                WHEN ps.end_date < NOW() + INTERVAL '7 days' THEN 1
                ELSE 0
            END,
            p.created_at DESC
        "#
    )
        .bind(user_id)
        .fetch_all(pool(db))
        .await?;

    let overview: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "title": row.try_get::<String, _>("title").unwrap_or_default(),
            "price": row.try_get::<f64, _>("price").unwrap_or(0.0),
            "location": row.try_get::<String, _>("location").unwrap_or_default(),
            "property_status": row.try_get::<String, _>("property_status").unwrap_or_default(),
            "plan_name": row.try_get::<String, _>("plan_name").unwrap_or_default(),
            "plan_tier": row.try_get::<String, _>("plan_tier").unwrap_or_default(),
            "plan_price": row.try_get::<f64, _>("plan_price").unwrap_or(0.0),
            "sub_status": row.try_get::<String, _>("sub_status").unwrap_or("none".parse().unwrap()),
            "days_remaining": row.try_get::<i32, _>("days_remaining").unwrap_or(0),
            "start_date": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("start_date")
                .ok().flatten().map(|d| d.to_string()),
            "end_date": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("end_date")
                .ok().flatten().map(|d| d.to_string()),
        })
    }).collect();

    Ok(overview)
}

// Add this constant at the top of the file (near the other constants if any)
pub const SUBSCRIPTION_COMMISSION_RATE: f64 = 10.0; // 10% to the converting agent

// ───────────────────────────────────────────
// Subscribe a Property to a Plan (with M-Pesa payment + Agent Commission)
// ───────────────────────────────────────────
pub async fn subscribe_property(
    db: &rento_core::Database,
    email_service: &rento_core::email::EmailService,
    mpesa_client: &crate::services::mpesa::MpesaClient,
    user_id: &Uuid,
    plan_id: &str,
    property_id: &str,
    phone: &str,
) -> ApiResult<serde_json::Value> {
    let plan_uuid = Uuid::parse_str(plan_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid plan ID: {}", e)))?;
    let property_uuid = Uuid::parse_str(property_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid property ID: {}", e)))?;

    // 1. Verify user owns the property
    let owner_check: Option<Uuid> = sqlx::query_scalar(
        "SELECT owner_id FROM properties WHERE id = $1"
    )
        .bind(property_uuid)
        .fetch_optional(pool(db))
        .await?;

    match owner_check {
        Some(id) if id == *user_id => {},
        Some(_) => return Err(ApiError::BadRequest("Property does not belong to you".into())),
        None => return Err(ApiError::NotFound("Property not found".into())),
    }

    // 2. Get plan details
    let plan: Option<(String, f64, String)> = sqlx::query_as(
        "SELECT name, price::float8, duration::text FROM subscription_plans WHERE id = $1 AND is_active = true"
    )
        .bind(plan_uuid)
        .fetch_optional(pool(db))
        .await?;

    let (plan_name, price, duration) = match plan {
        Some(p) => p,
        None => return Err(ApiError::NotFound("Plan not found or inactive".into())),
    };

    // 3. Simulate M-Pesa payment
    let account_ref = format!("RENTO-SUB-{}", &property_uuid.to_string()[..8]);
    let (merchant_request_id, checkout_request_id, receipt_number) =
        mpesa_client.simulate_payment(pool(db), phone, price as u32, &account_ref).await?;

    // 4. Find the M-Pesa transaction
    let mpesa_tx_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM mpesa_transactions WHERE checkout_request_id = $1"
    )
        .bind(&checkout_request_id)
        .fetch_one(pool(db))
        .await?;

    // 5. Create payment record
    let payment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO payments
            (payer_id, mpesa_transaction_id, payment_type, reference_id, amount, status, paid_at)
        VALUES ($1, $2, 'subscription', $3, $4, 'completed', NOW())
        RETURNING id
        "#
    )
        .bind(user_id)
        .bind(mpesa_tx_id)
        .bind(property_uuid)
        .bind(price)
        .fetch_one(pool(db))
        .await?;

    // 6. Calculate end date based on duration
    let now = chrono::Utc::now();
    let end_date = match duration.as_str() {
        "monthly" => now + chrono::Duration::days(30),
        "quarterly" => now + chrono::Duration::days(90),
        "yearly" => now + chrono::Duration::days(365),
        "permanent" => now + chrono::Duration::days(36500),
        _ => now + chrono::Duration::days(30),
    };

    // 7. Create subscription record
    let subscription_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO property_subscriptions
            (id, property_id, plan_id, status, amount_paid, payment_status, start_date, end_date)
        VALUES ($1, $2, $3, 'active', $4, 'completed', $5, $6)
        "#
    )
        .bind(subscription_id)
        .bind(property_uuid)
        .bind(plan_uuid)
        .bind(price)
        .bind(now)
        .bind(end_date)
        .execute(pool(db))
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create subscription: {}", e)))?;

    // 8. Update property subscription status
    sqlx::query(
        "UPDATE properties SET subscription_status = 'active', subscription_tier = $1, subscription_start_date = $2, subscription_end_date = $3 WHERE id = $4"
    )
        .bind(&plan_name)
        .bind(now)
        .bind(end_date)
        .bind(property_uuid)
        .execute(pool(db))
        .await?;

    // 9. Send subscription confirmation email to property owner
    let owner_email: String = sqlx::query_scalar(
        "SELECT email FROM account_users WHERE id = $1"
    )
        .bind(user_id)
        .fetch_one(pool(db))
        .await?;

    let _ = email_service
        .send_subscription_confirmation(&owner_email, price, &plan_name, &receipt_number, &end_date.to_string())
        .await
        .map_err(|e| tracing::warn!("Failed to send subscription confirmation: {}", e));

    // 10. Find the converting agent and credit 10% commission
    let agent_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT agent_id FROM agent_conversions WHERE property_owner_id = $1"
    )
        .bind(user_id)
        .fetch_optional(pool(db))
        .await?;

    let mut agent_commission_credited = 0.0;

    if let Some(agent_id) = agent_id {
        let commission_amount = price * (SUBSCRIPTION_COMMISSION_RATE / 100.0);

        // Ensure wallet exists
        crate::services::wallet::get_or_create_wallet(pool(db), &agent_id).await?;

        let mut tx = pool(db).begin().await?;

        // Create ledger entry
        let ledger_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO commission_ledger
                (agent_id, payment_id, property_owner_id, property_id,
                 commission_type, gross_amount, commission_rate, commission_amount,
                 status, credited_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'credited', NOW())
            RETURNING id
            "#
        )
            .bind(agent_id)
            .bind(payment_id)
            .bind(user_id)
            .bind(property_uuid)
            .bind("subscription_10pct")
            .bind(price)
            .bind(SUBSCRIPTION_COMMISSION_RATE)
            .bind(commission_amount)
            .fetch_one(&mut *tx)
            .await?;

        // Credit wallet
        crate::services::wallet::credit_wallet(
            &mut tx,
            &agent_id,
            commission_amount,
            &ledger_id.to_string(),
            &format!("subscription_10pct commission on KES {:.2} subscription", price),
        )
            .await?;

        tx.commit().await?;
        agent_commission_credited = commission_amount;

        // Send commission email to agent
        let agent_email: String = sqlx::query_scalar(
            "SELECT email FROM account_users WHERE id = $1"
        )
            .bind(agent_id)
            .fetch_one(pool(db))
            .await?;

        let _ = email_service
            .send_commission_notification(&agent_email, commission_amount, price, "subscription_10pct")
            .await
            .map_err(|e| tracing::warn!("Failed to send commission email: {}", e));

        tracing::info!(
            "✅ Subscription commission: KES {:.2} credited to agent {} for subscription KES {:.2}",
            commission_amount, agent_id, price
        );
    }

    tracing::info!(
        "✅ Property {} subscribed to {} plan by owner {} (KES {:.2})",
        property_uuid, plan_name, user_id, price
    );

    Ok(serde_json::json!({
        "message": format!("Successfully subscribed to {} plan", plan_name),
        "subscription_id": subscription_id.to_string(),
        "plan_name": plan_name,
        "amount_paid": price,
        "end_date": end_date.to_string(),
        "receipt_number": receipt_number,
        "agent_commission": agent_commission_credited,
    }))
}

// ───────────────────────────────────────────
// Payment History for Property Owner
// ───────────────────────────────────────────
pub async fn get_payment_history(
    db: &rento_core::Database,
    user_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            p.id::text,
            p.payment_type,
            p.amount::float8,
            p.status,
            p.paid_at,
            p.reference_id::text,
            p.created_at,
            mt.mpesa_receipt_number,
            mt.phone_number,
            -- For subscription payments, get the property title
            CASE
                WHEN p.payment_type = 'subscription' AND p.reference_id IS NOT NULL THEN
                    (SELECT title FROM properties WHERE id = p.reference_id)
                ELSE NULL
            END as property_title,
            -- For subscription payments, get the plan name
            CASE
                WHEN p.payment_type = 'subscription' AND p.reference_id IS NOT NULL THEN
                    (SELECT sp.name FROM property_subscriptions ps
                     JOIN subscription_plans sp ON ps.plan_id = sp.id
                     WHERE ps.property_id = p.reference_id
                     ORDER BY ps.created_at DESC LIMIT 1)
                ELSE NULL
            END as plan_name
        FROM payments p
        LEFT JOIN mpesa_transactions mt ON p.mpesa_transaction_id = mt.id
        WHERE p.payer_id = $1
        ORDER BY p.paid_at DESC NULLS LAST, p.created_at DESC
        "#
    )
        .bind(user_id)
        .fetch_all(pool(db))
        .await?;

    let history: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        let payment_type: String = row.try_get("payment_type").unwrap_or_default();
        let amount: f64 = row.try_get::<f64, _>("amount").unwrap_or(0.0);
        let status: String = row.try_get("status").unwrap_or_default();
        let receipt: Option<String> = row.try_get("mpesa_receipt_number").ok().flatten();
        let phone: Option<String> = row.try_get("phone_number").ok().flatten();
        let property_title: Option<String> = row.try_get("property_title").ok().flatten();
        let plan_name: Option<String> = row.try_get("plan_name").ok().flatten();
        let paid_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("paid_at").ok().flatten();
        let created_at: chrono::DateTime<chrono::Utc> = row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
            .unwrap_or_else(|_| chrono::Utc::now());

        // Human-readable description
        let description = match payment_type.as_str() {
            "registration_fee" => "Registration Fee".to_string(),
            "subscription" => {
                let plan = plan_name.as_deref().unwrap_or("Subscription");
                let prop = property_title.as_deref().unwrap_or("Property");
                format!("{} — {}", plan, prop)
            }
            "renewal" => "Subscription Renewal".to_string(),
            _ => payment_type.clone(),
        };

        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "payment_type": payment_type,
            "description": description,
            "amount": amount,
            "status": status,
            "receipt_number": receipt,
            "phone_number": phone,
            "property_title": property_title,
            "plan_name": plan_name,
            "paid_at": paid_at.map(|d| d.to_string()),
            "created_at": created_at.to_string(),
        })
    }).collect();

    Ok(history)
}

// ───────────────────────────────────────────
// Payment Summary (totals for the dashboard)
// ───────────────────────────────────────────
pub async fn get_payment_summary(
    db: &rento_core::Database,
    user_id: &Uuid,
) -> ApiResult<serde_json::Value> {
    let total_paid: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount)::float8, 0) FROM payments WHERE payer_id = $1 AND status = 'completed'"
    )
        .bind(user_id)
        .fetch_one(pool(db))
        .await?;

    let total_payments: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payments WHERE payer_id = $1 AND status = 'completed'"
    )
        .bind(user_id)
        .fetch_one(pool(db))
        .await?;

    let has_paid_registration_fee = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM payments WHERE payer_id = $1 AND payment_type = 'registration_fee' AND status = 'completed')"
    )
        .bind(user_id)
        .fetch_one(pool(db))
        .await?;

    let active_subscriptions: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM property_subscriptions ps
        JOIN properties p ON ps.property_id = p.id
        WHERE p.owner_id = $1 AND ps.status = 'active' AND ps.end_date > NOW()
        "#
    )
        .bind(user_id)
        .fetch_one(pool(db))
        .await?;

    Ok(serde_json::json!({
        "total_paid": total_paid,
        "total_payments": total_payments,
        "has_paid_registration_fee": has_paid_registration_fee,
        "active_subscriptions": active_subscriptions,
    }))
}
// ───────────────────────────────────────────
// Payout Management
// ───────────────────────────────────────────

pub const MINIMUM_PAYOUT_AMOUNT: f64 = 500.0; // Minimum KES 500 to request payout

/// Agent requests a payout from their wallet
pub async fn request_payout(
    db: &rento_core::Database,
    agent_id: &Uuid,
    amount: f64,
    mpesa_phone: &str,
) -> ApiResult<serde_json::Value> {
    // 1. Validate amount
    if amount < MINIMUM_PAYOUT_AMOUNT {
        return Err(ApiError::BadRequest(format!(
            "Minimum payout amount is KES {:.0}",
            MINIMUM_PAYOUT_AMOUNT
        )));
    }

    // 2. Validate phone number
    let digits: String = mpesa_phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if !((digits.starts_with("254") && digits.len() == 12)
        || (digits.starts_with("0") && digits.len() == 10)
        || (digits.starts_with("7") && digits.len() == 9))
    {
        return Err(ApiError::BadRequest("Invalid M-Pesa phone number".into()));
    }

    // Normalize phone to 254 format
    let normalized_phone = if digits.starts_with("0") {
        format!("254{}", &digits[1..])
    } else if digits.starts_with("7") && digits.len() == 9 {
        format!("254{}", digits)
    } else {
        digits
    };

    // 3. Check wallet balance
    let wallet = crate::services::wallet::get_or_create_wallet(pool(db), agent_id).await?;
    if wallet.balance < amount {
        return Err(ApiError::BadRequest(format!(
            "Insufficient balance. Available: KES {:.2}, Requested: KES {:.2}",
            wallet.balance, amount
        )));
    }

    // 4. Check for existing pending payout
    let existing_pending: Option<String> = sqlx::query_scalar(
        "SELECT id::text FROM payout_requests WHERE agent_id = $1 AND status = 'pending' LIMIT 1"
    )
        .bind(agent_id)
        .fetch_optional(pool(db))
        .await?;

    if existing_pending.is_some() {
        return Err(ApiError::BadRequest(
            "You already have a pending payout request. Please wait for it to be processed.".into()
        ));
    }

    // 5. Create payout request
    let payout_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO payout_requests (agent_id, amount, mpesa_phone, status)
        VALUES ($1, $2, $3, 'pending')
        RETURNING id
        "#
    )
        .bind(agent_id)
        .bind(amount)
        .bind(&normalized_phone)
        .fetch_one(pool(db))
        .await?;

    // 6. Debit wallet (move funds to pending)
    let mut tx = pool(db).begin().await?;
    crate::services::wallet::debit_wallet(
        &mut tx,
        agent_id,
        amount,
        &payout_id.to_string(),
        &format!("Payout request #{} to {}", &payout_id.to_string()[..8], normalized_phone),
    )
        .await?;

    // 7. Update pending balance
    sqlx::query(
        "UPDATE agent_wallets SET pending_balance = pending_balance + $1, updated_at = NOW() WHERE agent_id = $2"
    )
        .bind(amount)
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    tracing::info!(
        "💸 Payout request created: {} for KES {:.2} by agent {}",
        payout_id, amount, agent_id
    );

    Ok(serde_json::json!({
        "message": format!("Payout request of KES {:.2} submitted successfully", amount),
        "payout_id": payout_id.to_string(),
        "amount": amount,
        "mpesa_phone": normalized_phone,
        "status": "pending"
    }))
}

/// Get agent's payout history
pub async fn get_agent_payout_history(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            pr.id::text,
            pr.amount::float8,
            pr.status,
            pr.mpesa_phone,
            pr.created_at,
            pr.processed_at,
            pr.admin_notes
        FROM payout_requests pr
        WHERE pr.agent_id = $1
        ORDER BY pr.created_at DESC
        LIMIT 100
        "#
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await?;

    let history: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "amount": row.try_get::<f64, _>("amount").unwrap_or(0.0),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "mpesa_phone": row.try_get::<String, _>("mpesa_phone").unwrap_or_default(),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
            "processed_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("processed_at")
                .ok().flatten().map(|d| d.to_string()),
            "admin_notes": row.try_get::<Option<String>, _>("admin_notes").ok().flatten(),
        })
    }).collect();

    Ok(history)
}

/// Get all payout history (admin view)
pub async fn get_all_payout_history(
    db: &rento_core::Database,
    status_filter: Option<&str>,
) -> ApiResult<Vec<serde_json::Value>> {
    let query = if let Some(status) = status_filter {
        format!(
            r#"
            SELECT
                pr.id::text, pr.amount::float8, pr.status, pr.mpesa_phone,
                pr.created_at, pr.processed_at, pr.admin_notes,
                u.id::text as agent_id,
                COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name,
                u.email as agent_email
            FROM payout_requests pr
            JOIN account_users u ON pr.agent_id = u.id
            WHERE pr.status = '{}'
            ORDER BY pr.created_at DESC
            LIMIT 200
            "#,
            status
        )
    } else {
        r#"
        SELECT
            pr.id::text, pr.amount::float8, pr.status, pr.mpesa_phone,
            pr.created_at, pr.processed_at, pr.admin_notes,
            u.id::text as agent_id,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name,
            u.email as agent_email
        FROM payout_requests pr
        JOIN account_users u ON pr.agent_id = u.id
        ORDER BY pr.created_at DESC
        LIMIT 200
        "#.to_string()
    };

    let rows = sqlx::query(&query)
        .fetch_all(pool(db))
        .await?;

    let payouts: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "amount": row.try_get::<f64, _>("amount").unwrap_or(0.0),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "mpesa_phone": row.try_get::<String, _>("mpesa_phone").unwrap_or_default(),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
            "processed_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("processed_at")
                .ok().flatten().map(|d| d.to_string()),
            "admin_notes": row.try_get::<Option<String>, _>("admin_notes").ok().flatten(),
            "agent_id": row.try_get::<String, _>("agent_id").unwrap_or_default(),
            "agent_name": row.try_get::<String, _>("agent_name").unwrap_or_default(),
            "agent_email": row.try_get::<String, _>("agent_email").unwrap_or_default(),
        })
    }).collect();

    Ok(payouts)
}

/// Get payout statistics for admin dashboard
pub async fn get_payout_stats(db: &rento_core::Database) -> ApiResult<serde_json::Value> {
    let pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payout_requests WHERE status = 'pending'"
    )
        .fetch_one(pool(db))
        .await?;

    let pending_amount: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount)::float8, 0) FROM payout_requests WHERE status = 'pending'"
    )
        .fetch_one(pool(db))
        .await?;

    let approved_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payout_requests WHERE status = 'approved'"
    )
        .fetch_one(pool(db))
        .await?;

    let approved_amount: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount)::float8, 0) FROM payout_requests WHERE status = 'approved'"
    )
        .fetch_one(pool(db))
        .await?;

    let rejected_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payout_requests WHERE status = 'rejected'"
    )
        .fetch_one(pool(db))
        .await?;

    Ok(serde_json::json!({
        "pending_count": pending_count,
        "pending_amount": pending_amount,
        "approved_count": approved_count,
        "approved_amount": approved_amount,
        "rejected_count": rejected_count,
        "total_processed": approved_count + rejected_count,
    }))
}
// ───────────────────────────────────────────
// Owner Inquiry Management
// ───────────────────────────────────────────

pub async fn get_owner_inquiries(db: &rento_core::Database, owner_id: &Uuid) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            i.id::text, i.name as inquirer_name, i.email as inquirer_email, i.phone as inquirer_phone,
            i.message, i.status, i.created_at,
            p.id::text as property_id, p.title as property_title
        FROM admin_inquiries i
        JOIN properties p ON i.property_id = p.id
        WHERE p.owner_id = $1
        ORDER BY i.created_at DESC
        "#
    )
        .bind(owner_id)
        .fetch_all(pool(db))
        .await?;

    let inquiries: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "inquirer_name": row.try_get::<String, _>("inquirer_name").unwrap_or_default(),
            "inquirer_email": row.try_get::<String, _>("inquirer_email").unwrap_or_default(),
            "inquirer_phone": row.try_get::<Option<String>, _>("inquirer_phone").ok().flatten(),
            "message": row.try_get::<String, _>("message").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
            "property_id": row.try_get::<String, _>("property_id").unwrap_or_default(),
            "property_title": row.try_get::<String, _>("property_title").unwrap_or_default(),
        })
    }).collect();

    Ok(inquiries)
}

pub async fn update_owner_inquiry_status(
    db: &rento_core::Database,
    owner_id: &Uuid,
    inquiry_id: &str,
    new_status: &str,
) -> ApiResult<()> {
    let id = Uuid::parse_str(inquiry_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // Verify ownership before updating
    let is_owner: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM admin_inquiries i
            JOIN properties p ON i.property_id = p.id
            WHERE i.id = $1 AND p.owner_id = $2
        )
        "#
    )
        .bind(id)
        .bind(owner_id)
        .fetch_one(pool(db))
        .await?;

    if !is_owner {
        return Err(ApiError::Unauthorized("You do not own the property associated with this inquiry".into()));
    }

    sqlx::query("UPDATE admin_inquiries SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_status)
        .bind(id)
        .execute(pool(db))
        .await?;

    Ok(())
}

// ───────────────────────────────────────────
// Agent Lead Pipeline Management
// ───────────────────────────────────────────


pub async fn get_agent_leads(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            al.id::text,
            al.full_name as client_name,
            al.email as client_email,
            al.phone as client_phone,
            al.status::text as lead_status,
            COALESCE(al.pipeline_stage, 'new') as pipeline_stage,
            al.created_at,
            al.updated_at
        FROM agent_leads al
        WHERE al.claimed_by = $1
        ORDER BY
            CASE COALESCE(al.pipeline_stage, 'new')
                WHEN 'new' THEN 1
                WHEN 'contacted' THEN 2
                WHEN 'viewing_scheduled' THEN 3
                WHEN 'negotiation' THEN 4
                WHEN 'closed' THEN 5
                WHEN 'lost' THEN 6
                ELSE 7
            END,
            al.created_at DESC
        "#
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch agent leads: {}", e);
            ApiError::Internal(format!("Database error: {}", e))
        })?;

    let leads: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        let client_name: String = row.try_get::<String, _>("client_name").unwrap_or_default();
        let client_email: String = row.try_get::<String, _>("client_email").unwrap_or_default();
        let client_phone: Option<String> = row.try_get::<Option<String>, _>("client_phone").ok().flatten();
        let lead_status: String = row.try_get::<String, _>("lead_status").unwrap_or_default();

        // Build a descriptive "property_title" from the lead status since there's no property_interest column
        let property_title = match lead_status.as_str() {
            "pending" => "General Lead".to_string(),
            "converted" => "Converted Client".to_string(),
            "lost" => "Lost Lead".to_string(),
            other => format!("Lead: {}", other),
        };

        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "client_name": client_name,
            "client_email": client_email,
            "client_phone": client_phone,
            "property_interest": null,
            "property_title": property_title,
            "notes": null,
            "lead_status": lead_status,
            "pipeline_stage": row.try_get::<String, _>("pipeline_stage").unwrap_or_else(|_| "new".to_string()),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
        })
    }).collect();

    Ok(leads)
}

pub async fn update_lead_stage(
    db: &rento_core::Database,
    agent_id: &Uuid,
    lead_id: &str,
    new_stage: &str,
) -> ApiResult<()> {
    let id = Uuid::parse_str(lead_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // ✅ FIX: Use `claimed_by` instead of `agent_id`
    let is_owner: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM agent_leads WHERE id = $1 AND claimed_by = $2)"
    )
        .bind(id)
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    if !is_owner {
        return Err(ApiError::Unauthorized("You do not own this lead".into()));
    }

    sqlx::query(
        "UPDATE agent_leads SET pipeline_stage = $1, updated_at = NOW() WHERE id = $2"
    )
        .bind(new_stage)
        .bind(id)
        .execute(pool(db))
        .await?;

    Ok(())
}

// ═══════════════════════════════════════════
// FEATURE 2: AGENT PERFORMANCE DASHBOARD
// ✅ UPDATED: Includes virtual tour fee earnings
// ═══════════════════════════════════════════
pub async fn get_agent_performance(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<serde_json::Value> {
    // ─── LEADS ───
    let total_leads: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_leads WHERE claimed_by = $1"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    let converted_leads: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_leads WHERE claimed_by = $1 AND pipeline_stage = 'closed'"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    let active_leads: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_leads WHERE claimed_by = $1 AND pipeline_stage NOT IN ('closed', 'lost')"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    let conversion_rate = if total_leads > 0 {
        (converted_leads as f64 / total_leads as f64) * 100.0
    } else {
        0.0
    };

    // ─── WALLET (includes ALL earnings: handshake + subscription + tour fees + bonuses) ───
    let wallet = crate::services::wallet::get_or_create_wallet(pool(db), agent_id).await?;

    // ─── COMMISSIONS THIS MONTH ───
    let commissions_this_month: f64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(commission_amount)::float8, 0)
        FROM commission_ledger
        WHERE agent_id = $1
          AND created_at >= date_trunc('month', NOW())
          AND status = 'credited'
        "#
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    let commissions_count_month: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM commission_ledger
        WHERE agent_id = $1
          AND created_at >= date_trunc('month', NOW())
          AND status = 'credited'
        "#
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    // ─── PROPERTIES & CONVERSIONS ───
    let properties_managed: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT p.id)
        FROM properties p
        JOIN agent_conversions ac ON ac.property_owner_id = p.owner_id
        WHERE ac.agent_id = $1
        "#
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    let owners_converted: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_conversions WHERE agent_id = $1"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    // ─── REFERRALS ───
    let referrals_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_referrals WHERE agent_id = $1"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    let referrals_completed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_referrals WHERE agent_id = $1 AND signup_completed = TRUE"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    // ═══════════════════════════════════════════
    // ✅ NEW: VIRTUAL TOUR FEE EARNINGS
    // ═══════════════════════════════════════════

    // Tour fee earnings from commission_ledger
    let tour_stats: Option<(i64, f64)> = sqlx::query_as(
        r#"
        SELECT COUNT(*), COALESCE(SUM(commission_amount)::float8, 0)
        FROM commission_ledger
        WHERE agent_id = $1 AND commission_type = 'tour_fee' AND status = 'credited'
        "#
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await
        .ok();

    let (tours_completed, tour_fee_total) = tour_stats.unwrap_or((0, 0.0));

    // Tour fee earnings THIS MONTH
    let tour_fees_this_month: f64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(commission_amount)::float8, 0)
        FROM commission_ledger
        WHERE agent_id = $1
          AND commission_type = 'tour_fee'
          AND status = 'credited'
          AND created_at >= date_trunc('month', NOW())
        "#
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await
        .unwrap_or(0.0);

    // ─── SLA PERFORMANCE ───
    let sla_metrics: Option<(i32, i32, i32, i32, i32)> = sqlx::query_as(
        r#"
        SELECT
            total_tours_assigned,
            tours_fulfilled_on_time,
            tours_fulfilled_late,
            tours_expired,
            average_fulfillment_minutes
        FROM agent_sla_metrics
        WHERE agent_id = $1
        "#
    )
        .bind(agent_id)
        .fetch_optional(pool(db))
        .await?;

    let (tours_assigned, tours_on_time, tours_late, tours_expired, avg_fulfillment_mins) =
        sla_metrics.unwrap_or((0, 0, 0, 0, 0));

    let total_fulfilled = tours_on_time + tours_late;
    let on_time_rate = if total_fulfilled > 0 {
        (tours_on_time as f64 / total_fulfilled as f64 * 100.0).round() as i32
    } else {
        0
    };

    // ═══════════════════════════════════════════
    // ✅ NEW: EARNINGS BREAKDOWN BY TYPE
    // ═══════════════════════════════════════════
    let breakdown_rows: Vec<(String, f64, i64)> = sqlx::query_as(
        r#"
        SELECT
            commission_type,
            COALESCE(SUM(commission_amount)::float8, 0) as total,
            COUNT(*) as count
        FROM commission_ledger
        WHERE agent_id = $1 AND status = 'credited'
        GROUP BY commission_type
        ORDER BY total DESC
        "#
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await?;

    let earnings_breakdown: Vec<serde_json::Value> = breakdown_rows.iter().map(|(ctype, total, count)| {
        let label = match ctype.as_str() {
            "handshake_30pct" => "Registration Fee Commission (30%)",
            "subscription_10pct" => "Subscription Commission (10%)",
            "tour_fee" => "Virtual Tour Fee (KES 20/tour)",
            "referral_bonus" => "Referral Bonus",
            other => other,
        };
        let icon = match ctype.as_str() {
            "handshake_30pct" => "🤝",
            "subscription_10pct" => "⭐",
            "tour_fee" => "🎬",
            "referral_bonus" => "🏆",
            _ => "💰",
        };
        serde_json::json!({
            "type": ctype,
            "label": label,
            "icon": icon,
            "total": total,
            "count": count,
        })
    }).collect();

    // ─── DAILY ACTIVITY (last 7 days) ───
    let daily_activity = sqlx::query(
        r#"
        SELECT
            DATE(created_at) as day,
            COALESCE(SUM(commission_amount)::float8, 0) as total
        FROM commission_ledger
        WHERE agent_id = $1
          AND created_at >= NOW() - INTERVAL '7 days'
          AND status = 'credited'
        GROUP BY DATE(created_at)
        ORDER BY day ASC
        "#
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await?;

    let daily_data: Vec<serde_json::Value> = daily_activity.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "day": row.try_get::<chrono::NaiveDate, _>("day").map(|d| d.to_string()).unwrap_or_default(),
            "total": row.try_get::<f64, _>("total").unwrap_or(0.0),
        })
    }).collect();

    // ─── PAYOUT ELIGIBILITY ───
    let can_request_payout = wallet.balance >= MINIMUM_PAYOUT_AMOUNT;

    Ok(serde_json::json!({
        // Leads
        "total_leads": total_leads,
        "converted_leads": converted_leads,
        "active_leads": active_leads,
        "conversion_rate": conversion_rate,

        // Wallet (includes ALL earnings: handshake + subscription + tour fees + bonuses)
        "total_earned": wallet.total_earned,
        "current_balance": wallet.balance,
        "pending_balance": wallet.pending_balance,
        "total_withdrawn": wallet.total_withdrawn,
        "can_request_payout": can_request_payout,
        "minimum_payout": MINIMUM_PAYOUT_AMOUNT,

        // Commissions this month
        "commissions_this_month": commissions_this_month,
        "commissions_count_month": commissions_count_month,

        // Properties & conversions
        "properties_managed": properties_managed,
        "owners_converted": owners_converted,

        // Referrals
        "referrals_count": referrals_count,
        "referrals_completed": referrals_completed,

        // ✅ NEW: Virtual Tour Earnings
        "tour_fee_earnings": {
            "tours_completed": tours_completed,
            "total_earned": tour_fee_total,
            "fee_per_tour": TOUR_FEE_KES,
            "this_month": tour_fees_this_month,
        },

        // ✅ NEW: SLA Performance
        "sla_performance": {
            "total_assigned": tours_assigned,
            "on_time": tours_on_time,
            "late": tours_late,
            "expired": tours_expired,
            "on_time_rate_percent": on_time_rate,
            "avg_fulfillment_minutes": avg_fulfillment_mins,
        },

        // ✅ NEW: Earnings breakdown by type
        "earnings_breakdown": earnings_breakdown,

        // Daily activity
        "daily_activity": daily_data,
    }))
}

// ═══════════════════════════════════════════
// FEATURE 3: REFERRAL LINKS
// ═══════════════════════════════════════════

pub async fn record_referral_signup(
    db: &rento_core::Database,
    agent_id: &Uuid,
    referred_email: &str,
    referred_name: Option<&str>,
) -> ApiResult<serde_json::Value> {
    // Check if already referred
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM agent_referrals WHERE agent_id = $1 AND referred_email = $2"
    )
        .bind(agent_id)
        .bind(referred_email)
        .fetch_optional(pool(db))
        .await?;

    if existing.is_some() {
        return Err(ApiError::BadRequest("This email has already been referred by this agent".into()));
    }

    let referral_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO agent_referrals (agent_id, referred_email, referred_name, signup_completed)
        VALUES ($1, $2, $3, TRUE)
        RETURNING id
        "#
    )
        .bind(agent_id)
        .bind(referred_email)
        .bind(referred_name)
        .fetch_one(pool(db))
        .await?;

    Ok(serde_json::json!({
        "message": "Referral recorded successfully",
        "referral_id": referral_id.to_string(),
    }))
}

pub async fn get_agent_referrals(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            ar.id::text,
            ar.referred_email,
            ar.referred_name,
            ar.signup_completed,
            ar.conversion_completed,
            ar.created_at,
            ar.converted_at,
            u.id::text as user_id
        FROM agent_referrals ar
        LEFT JOIN account_users u ON ar.referred_user_id = u.id
        WHERE ar.agent_id = $1
        ORDER BY ar.created_at DESC
        "#
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await?;

    let referrals: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "referred_email": row.try_get::<String, _>("referred_email").unwrap_or_default(),
            "referred_name": row.try_get::<Option<String>, _>("referred_name").ok().flatten(),
            "signup_completed": row.try_get::<bool, _>("signup_completed").unwrap_or(false),
            "conversion_completed": row.try_get::<bool, _>("conversion_completed").unwrap_or(false),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
            "converted_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("converted_at")
                .ok().flatten().map(|d| d.to_string()),
        })
    }).collect();

    Ok(referrals)
}

// ═══════════════════════════════════════════
// FEATURE 4: B2C PAYOUT AUTOMATION
// ═══════════════════════════════════════════

pub async fn process_approved_payout_b2c(
    db: &rento_core::Database,
    payout_id: &str,
) -> ApiResult<serde_json::Value> {
    let id = Uuid::parse_str(payout_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // Get payout details
    let payout: Option<(Uuid, Uuid, f64, String, String)> = sqlx::query_as(
        r#"
        SELECT id, agent_id, amount::float8, status, mpesa_phone
        FROM payout_requests WHERE id = $1
        "#
    )
        .bind(id)
        .fetch_optional(pool(db))
        .await?;

    let (payout_uuid, agent_id, amount, status, phone) = match payout {
        Some(p) => p,
        None => return Err(ApiError::NotFound("Payout not found".into())),
    };

    if status != "approved" {
        return Err(ApiError::BadRequest("Payout must be approved before B2C processing".into()));
    }

    // Check if already processed
    let existing_b2c: Option<String> = sqlx::query_scalar(
        "SELECT status FROM b2c_payouts WHERE payout_request_id = $1"
    )
        .bind(id)
        .fetch_optional(pool(db))
        .await?;

    if let Some(s) = existing_b2c {
        if s == "delivered" || s == "sent" {
            return Err(ApiError::BadRequest("Payout already processed".into()));
        }
    }

    // Simulate B2C call (in production, this would call Safaricom Daraja B2C API)
    let conversation_id = format!("B2C-{}", &Uuid::new_v4().to_string()[..8]);
    let originator_id = format!("ORI-{}", &Uuid::new_v4().to_string()[..8]);

    // Create B2C record
    let b2c_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO b2c_payouts
            (payout_request_id, agent_id, amount, phone_number, status,
             conversation_id, originator_conversation_id, result_code,
             result_description, last_attempt_at, completed_at)
        VALUES ($1, $2, $3, $4, 'delivered', $5, $6, '0', 'Accepted delivery', NOW(), NOW())
        RETURNING id
        "#
    )
        .bind(id)
        .bind(agent_id)
        .bind(amount)
        .bind(&phone)
        .bind(&conversation_id)
        .bind(&originator_id)
        .fetch_one(pool(db))
        .await?;

    // Update wallet withdrawn total
    sqlx::query(
        "UPDATE agent_wallets SET total_withdrawn = total_withdrawn + $1, updated_at = NOW() WHERE agent_id = $2"
    )
        .bind(amount)
        .bind(agent_id)
        .execute(pool(db))
        .await?;

    tracing::info!(
        "💸 B2C payout {} delivered: KES {:.2} to {} (agent {})",
        b2c_id, amount, phone, agent_id
    );

    Ok(serde_json::json!({
        "message": format!("B2C payout of KES {:.2} delivered to {}", amount, phone),
        "b2c_id": b2c_id.to_string(),
        "conversation_id": conversation_id,
        "status": "delivered",
    }))
}

pub async fn get_b2c_payout_history(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            b.id::text,
            b.amount::float8,
            b.phone_number,
            b.status,
            b.conversation_id,
            b.result_description,
            b.created_at,
            b.completed_at,
            b.retry_count,
            pr.id::text as payout_request_id
        FROM b2c_payouts b
        JOIN payout_requests pr ON b.payout_request_id = pr.id
        WHERE b.agent_id = $1
        ORDER BY b.created_at DESC
        LIMIT 50
        "#
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await?;

    let history: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "amount": row.try_get::<f64, _>("amount").unwrap_or(0.0),
            "phone_number": row.try_get::<String, _>("phone_number").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "conversation_id": row.try_get::<Option<String>, _>("conversation_id").ok().flatten(),
            "result_description": row.try_get::<Option<String>, _>("result_description").ok().flatten(),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
            "completed_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
                .ok().flatten().map(|d| d.to_string()),
            "retry_count": row.try_get::<i32, _>("retry_count").unwrap_or(0),
            "payout_request_id": row.try_get::<String, _>("payout_request_id").unwrap_or_default(),
        })
    }).collect();

    Ok(history)
}

// ═══════════════════════════════════════════
// REFERRAL BONUS TIERS
// ═══════════════════════════════════════════

pub async fn get_bonus_tiers(db: &rento_core::Database) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        "SELECT id, tier_name, min_referrals, bonus_amount::float8, is_active FROM referral_bonus_tiers WHERE is_active = TRUE ORDER BY min_referrals ASC"
    )
        .fetch_all(pool(db))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bonus tiers: {}", e);
            ApiError::Internal(format!("Database error: {}", e))
        })?;

    let tiers: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<i32, _>("id").unwrap_or(0),
            "tier_name": row.try_get::<String, _>("tier_name").unwrap_or_default(),
            "min_referrals": row.try_get::<i32, _>("min_referrals").unwrap_or(0),
            "bonus_amount": row.try_get::<f64, _>("bonus_amount").unwrap_or(0.0),
            "is_active": row.try_get::<bool, _>("is_active").unwrap_or(true),
        })
    }).collect();

    Ok(tiers)
}

pub async fn get_agent_bonus_progress(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<serde_json::Value> {
    // Get current referral count
    let referral_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_referrals WHERE agent_id = $1 AND signup_completed = TRUE"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await
        .unwrap_or(0);

    // Get all tiers
    let tiers = get_bonus_tiers(db).await?;

    // Get claimed bonuses
    let claimed_rows = sqlx::query(
        "SELECT tier_id, bonus_amount::float8, claimed_at FROM agent_bonus_claims WHERE agent_id = $1"
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await
        .unwrap_or_default();

    let claimed_tier_ids: Vec<i32> = claimed_rows.iter()
        .filter_map(|r| { use sqlx::Row; r.try_get::<i32, _>("tier_id").ok() })
        .collect();

    let total_bonuses_earned: f64 = claimed_rows.iter()
        .filter_map(|r| { use sqlx::Row; r.try_get::<f64, _>("bonus_amount").ok() })
        .sum();

    // Build progress for each tier
    let tier_progress: Vec<serde_json::Value> = tiers.iter().map(|tier| {
        let tier_id = tier.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let min_ref = tier.get("min_referrals").and_then(|v| v.as_i64()).unwrap_or(0);
        let is_claimed = claimed_tier_ids.contains(&tier_id);
        let progress_pct = if min_ref > 0 {
            ((referral_count as f64 / min_ref as f64) * 100.0).min(100.0)
        } else { 100.0 };

        serde_json::json!({
            "tier": tier,
            "is_claimed": is_claimed,
            "progress_percent": progress_pct,
            "referrals_needed": (min_ref - referral_count).max(0),
        })
    }).collect();

    // Find next unclaimed tier
    let next_tier = tiers.iter().find(|t| {
        let tid = t.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        !claimed_tier_ids.contains(&tid)
    }).cloned();

    Ok(serde_json::json!({
        "current_referrals": referral_count,
        "total_bonuses_earned": total_bonuses_earned,
        "tiers_claimed": claimed_tier_ids.len(),
        "total_tiers": tiers.len(),
        "tier_progress": tier_progress,
        "next_tier": next_tier,
    }))
}

pub async fn check_and_award_bonuses(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    // Get current referral count
    let referral_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_referrals WHERE agent_id = $1 AND signup_completed = TRUE"
    )
        .bind(agent_id)
        .fetch_one(pool(db))
        .await
        .unwrap_or(0);

    // Get all active tiers
    let tier_rows = sqlx::query(
        "SELECT id, tier_name, min_referrals, bonus_amount::float8 FROM referral_bonus_tiers WHERE is_active = TRUE ORDER BY min_referrals ASC"
    )
        .fetch_all(pool(db))
        .await?;

    // Get already claimed tier IDs
    let claimed_ids: Vec<i32> = sqlx::query_scalar(
        "SELECT tier_id FROM agent_bonus_claims WHERE agent_id = $1"
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await
        .unwrap_or_default();

    let mut newly_awarded: Vec<serde_json::Value> = Vec::new();

    for row in &tier_rows {
        use sqlx::Row;
        let tier_id: i32 = row.try_get("id").unwrap_or(0);
        let tier_name: String = row.try_get("tier_name").unwrap_or_default();
        let min_referrals: i64 = row.try_get("min_referrals").unwrap_or(0);
        let bonus_amount: f64 = row.try_get("bonus_amount").unwrap_or(0.0);

        // Skip if already claimed or not yet eligible
        if claimed_ids.contains(&tier_id) || referral_count < min_referrals {
            continue;
        }

        // Award the bonus!
        let mut tx = pool(db).begin().await?;

        // Record the claim
        sqlx::query(
            r#"
            INSERT INTO agent_bonus_claims (agent_id, tier_id, bonus_amount, referral_count_at_claim)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (agent_id, tier_id) DO NOTHING
            "#
        )
            .bind(agent_id)
            .bind(tier_id)
            .bind(bonus_amount)
            .bind(referral_count)
            .execute(&mut *tx)
            .await?;

        // Credit wallet
        crate::services::wallet::credit_wallet(
            &mut tx,
            agent_id,
            bonus_amount,
            &format!("bonus_tier_{}", tier_id),
            &format!("🏆 {} Referral Bonus ({} referrals)", tier_name, referral_count),
        )
            .await?;

        tx.commit().await?;

        tracing::info!(
            "🏆 Agent {} awarded {} bonus: KES {:.2} for {} referrals",
            agent_id, tier_name, bonus_amount, referral_count
        );

        newly_awarded.push(serde_json::json!({
            "tier_name": tier_name,
            "bonus_amount": bonus_amount,
            "referral_count": referral_count,
        }));
    }

    Ok(newly_awarded)
}

// ═══════════════════════════════════════════
// AGENT LEADERBOARD
// ═══════════════════════════════════════════

pub async fn get_leaderboard(
    db: &rento_core::Database,
    current_agent_id: Option<&Uuid>,
    limit: i64,
) -> ApiResult<serde_json::Value> {
    // Build leaderboard from live data
    let rows = sqlx::query(
        r#"
        SELECT
            u.id::text as agent_id,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as agent_name,
            -- Conversions
            (SELECT COUNT(*) FROM agent_conversions WHERE agent_id = u.id) as total_conversions,
            -- Commissions
            (SELECT COALESCE(SUM(commission_amount)::float8, 0) FROM commission_ledger WHERE agent_id = u.id AND status = 'credited') as total_commissions,
            -- Referrals
            (SELECT COUNT(*) FROM agent_referrals WHERE agent_id = u.id AND signup_completed = TRUE) as total_referrals,
            -- Properties managed
            (SELECT COUNT(DISTINCT p.id) FROM properties p JOIN agent_conversions ac ON ac.property_owner_id = p.owner_id WHERE ac.agent_id = u.id) as properties_managed,
            -- Leads closed
            (SELECT COUNT(*) FROM agent_leads WHERE claimed_by = u.id AND pipeline_stage = 'closed') as leads_closed
        FROM account_users u
        WHERE u.role = 'AGENT'
        ORDER BY
            (SELECT COALESCE(SUM(commission_amount)::float8, 0) FROM commission_ledger WHERE agent_id = u.id AND status = 'credited') DESC,
            (SELECT COUNT(*) FROM agent_conversions WHERE agent_id = u.id) DESC
        LIMIT $1
        "#
    )
        .bind(limit)
        .fetch_all(pool(db))
        .await
        .map_err(|e| {
            tracing::error!("Failed to build leaderboard: {}", e);
            ApiError::Internal(format!("Database error: {}", e))
        })?;

    let mut agents: Vec<serde_json::Value> = rows.into_iter().enumerate().map(|(idx, row)| {
        use sqlx::Row;
        let conversions: i64 = row.try_get("total_conversions").unwrap_or(0);
        let commissions: f64 = row.try_get("total_commissions").unwrap_or(0.0);
        let referrals: i64 = row.try_get("total_referrals").unwrap_or(0);
        let properties: i64 = row.try_get("properties_managed").unwrap_or(0);
        let leads: i64 = row.try_get("leads_closed").unwrap_or(0);

        // Weighted score: commissions (40%) + conversions (30%) + referrals (20%) + properties (10%)
        let score = (commissions * 0.4)
            + (conversions as f64 * 1000.0 * 0.3)
            + (referrals as f64 * 500.0 * 0.2)
            + (properties as f64 * 200.0 * 0.1);

        let agent_id_str: String = row.try_get("agent_id").unwrap_or_default();
        let is_current = current_agent_id
            .map(|id| id.to_string() == agent_id_str)
            .unwrap_or(false);

        serde_json::json!({
            "rank": idx + 1,
            "agent_id": agent_id_str,
            "agent_name": row.try_get::<String, _>("agent_name").unwrap_or_default(),
            "total_conversions": conversions,
            "total_commissions": commissions,
            "total_referrals": referrals,
            "properties_managed": properties,
            "leads_closed": leads,
            "score": score,
            "is_current_user": is_current,
        })
    }).collect();

    // Find current agent's rank if they're not in top N
    let mut my_rank: Option<serde_json::Value> = None;
    if let Some(current_id) = current_agent_id {
        if !agents.iter().any(|a| a.get("is_current_user").and_then(|v| v.as_bool()).unwrap_or(false)) {
            // Agent not in top N, fetch their stats separately
            let my_stats = sqlx::query(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM agent_conversions WHERE agent_id = $1) as total_conversions,
                    (SELECT COALESCE(SUM(commission_amount)::float8, 0) FROM commission_ledger WHERE agent_id = $1 AND status = 'credited') as total_commissions,
                    (SELECT COUNT(*) FROM agent_referrals WHERE agent_id = $1 AND signup_completed = TRUE) as total_referrals,
                    (SELECT COUNT(DISTINCT p.id) FROM properties p JOIN agent_conversions ac ON ac.property_owner_id = p.owner_id WHERE ac.agent_id = $1) as properties_managed
                "#
            )
                .bind(current_id)
                .fetch_one(pool(db))
                .await;

            if let Ok(row) = my_stats {
                use sqlx::Row;
                let commissions: f64 = row.try_get("total_commissions").unwrap_or(0.0);
                let conversions: i64 = row.try_get("total_conversions").unwrap_or(0);
                let referrals: i64 = row.try_get("total_referrals").unwrap_or(0);
                let properties: i64 = row.try_get("properties_managed").unwrap_or(0);
                let score = (commissions * 0.4) + (conversions as f64 * 1000.0 * 0.3) + (referrals as f64 * 500.0 * 0.2) + (properties as f64 * 200.0 * 0.1);

                // Count how many agents have a higher score
                let rank: i64 = sqlx::query_scalar(
                    r#"
                    SELECT COUNT(*) + 1 FROM account_users u
                    WHERE u.role = 'AGENT' AND u.id != $1
                    AND (SELECT COALESCE(SUM(commission_amount)::float8, 0) FROM commission_ledger WHERE agent_id = u.id AND status = 'credited') > $2
                    "#
                )
                    .bind(current_id)
                    .bind(commissions)
                    .fetch_one(pool(db))
                    .await
                    .unwrap_or(0);

                let my_name: String = sqlx::query_scalar(
                    "SELECT COALESCE(NULLIF(first_name || ' ' || last_name, ' '), username) FROM account_users WHERE id = $1"
                )
                    .bind(current_id)
                    .fetch_one(pool(db))
                    .await
                    .unwrap_or_else(|_| "You".to_string());

                my_rank = Some(serde_json::json!({
                    "rank": rank,
                    "agent_name": my_name,
                    "total_commissions": commissions,
                    "total_conversions": conversions,
                    "total_referrals": referrals,
                    "properties_managed": properties,
                    "score": score,
                    "is_current_user": true,
                }));
            }
        }
    }

    let total_agents: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM account_users WHERE role = 'AGENT'"
    )
        .fetch_one(pool(db))
        .await
        .unwrap_or(0);

    Ok(serde_json::json!({
        "leaderboard": agents,
        "my_rank": my_rank,
        "total_agents": total_agents,
    }))
}

// ═══════════════════════════════════════════
// VIRTUAL TOUR SYSTEM
// ═══════════════════════════════════════════
pub const TOUR_FEE_KES: f64 = 20.00;
pub const TOUR_SLA_HOURS: i64 = 24;
pub const VIEWING_WINDOW_MINUTES: i64 = 120;

#[derive(Deserialize)]
pub struct UploadTourVideoRequest {
    pub tour_request_id: String,
    pub video_url: String,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i32>,
    pub file_size_bytes: Option<i64>,
    pub device_fingerprint: Option<String>,
    pub recording_started_at: Option<String>,
    pub recording_completed_at: Option<String>,
}

// ───────────────────────────────────────────
// 1. Request a Virtual Tour (Client side)
// ✅ MILESTONE 6: Sends email to client with payment instructions
// ───────────────────────────────────────────
pub async fn request_virtual_tour(
    db: &rento_core::Database,
    email_service: &rento_core::email::EmailService,  // ✅ NEW
    property_id: &str,
    client_email: &str,
    client_name: Option<&str>,
    client_phone: Option<&str>,
    client_id: Option<&Uuid>,
) -> ApiResult<serde_json::Value> {
    let prop_uuid = Uuid::parse_str(property_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid property ID: {}", e)))?;

    // Check property exists and is not delisted
    let property_check: Option<(bool, Uuid)> = sqlx::query_as(
        "SELECT COALESCE(is_delisted, FALSE), owner_id FROM properties WHERE id = $1"
    )
        .bind(prop_uuid)
        .fetch_optional(pool(db))
        .await?;

    let (is_delisted, owner_id) = match property_check {
        Some(p) => p,
        None => return Err(ApiError::NotFound("Property not found".into())),
    };

    if is_delisted {
        return Err(ApiError::BadRequest(
            "This property is no longer available for viewing".into()
        ));
    }

    // ✅ Get property title and location for the email
    let property_info: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT title, location FROM properties WHERE id = $1"
    )
        .bind(prop_uuid)
        .fetch_optional(pool(db))
        .await?;
    let (property_title, property_location) = property_info
        .unwrap_or_else(|| ("Unknown Property".to_string(), None));

    // Find the assigned agent (property owner's converting agent)
    let assigned_agent: Option<Uuid> = sqlx::query_scalar(
        "SELECT agent_id FROM agent_conversions WHERE property_owner_id = $1 LIMIT 1"
    )
        .bind(owner_id)
        .fetch_optional(pool(db))
        .await?;

    // Create tour request with 24-hour SLA
    let sla_deadline = chrono::Utc::now() + chrono::Duration::hours(TOUR_SLA_HOURS);

    let request_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO virtual_tour_requests
            (property_id, client_id, client_email, client_name, client_phone,
             fee_amount, assigned_agent_id, sla_deadline)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#
    )
        .bind(prop_uuid)
        .bind(client_id)
        .bind(client_email)
        .bind(client_name)
        .bind(client_phone)
        .bind(TOUR_FEE_KES)
        .bind(assigned_agent)
        .bind(sla_deadline)
        .fetch_one(pool(db))
        .await?;

    // ✅ SEND EMAIL to client with payment instructions
    let payment_instructions = format!(
        "Pay KES {:.2} via M-Pesa or bank transfer to activate your tour. \
         Our agent will record a fresh, watermarked video tour within 24 hours of payment confirmation.",
        TOUR_FEE_KES
    );
    let _ = email_service.send_tour_requested(
        client_email,
        client_name,
        &property_title,
        property_location.as_deref(),
        TOUR_FEE_KES,
        &request_id.to_string(),
        &payment_instructions,
    ).await.map_err(|e| tracing::warn!("⚠️ Failed to send tour requested email to {}: {}", client_email, e));

    tracing::info!(
        "🎬 Virtual tour requested: {} for property {} (SLA: {}h)",
        request_id, property_id, TOUR_SLA_HOURS
    );

    Ok(serde_json::json!({
        "request_id": request_id.to_string(),
        "fee_amount": TOUR_FEE_KES,
        "sla_deadline": sla_deadline.to_string(),
        "message": format!("Tour request created. Pay KES {:.2} to proceed.", TOUR_FEE_KES),
    }))
}

// ───────────────────────────────────────────
// 2. Confirm Payment for Tour
// ✅ MILESTONE 6: Sends email to agent with new assignment + SLA deadline
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 2. Confirm Payment for Tour
// ✅ MILESTONE 6: Sends email to agent with new assignment + SLA deadline
// ───────────────────────────────────────────
pub async fn confirm_tour_payment(
    db: &rento_core::Database,
    email_service: &rento_core::email::EmailService,
    request_id: &str,
    payment_reference: &str,
) -> ApiResult<serde_json::Value> {
    let id = Uuid::parse_str(request_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid request ID: {}", e)))?;

    // ✅ FIXED: Correct tuple types matching DB schema
    // (agent_id, client_name, property_title, property_location, agent_name, sla_deadline)
    let tour_details: Option<(Uuid, Option<String>, String, Option<String>, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT tr.assigned_agent_id, tr.client_name, p.title, p.location,
               COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username),
               tr.sla_deadline
        FROM virtual_tour_requests tr
        JOIN properties p ON tr.property_id = p.id
        LEFT JOIN account_users u ON tr.assigned_agent_id = u.id
        WHERE tr.id = $1
        "#
    )
        .bind(id)
        .fetch_optional(pool(db))
        .await?;

    sqlx::query(
        r#"
        UPDATE virtual_tour_requests
        SET fee_paid = TRUE,
            payment_reference = $1,
            status = 'pending',
            updated_at = NOW()
        WHERE id = $2 AND fee_paid = FALSE
        "#
    )
        .bind(payment_reference)
        .bind(id)
        .execute(pool(db))
        .await?;

    // ✅ SEND EMAIL to agent
    if let Some((agent_id, client_name, property_title, property_location, agent_name, sla_deadline)) = tour_details {
        let agent_email: Option<String> = sqlx::query_scalar(
            "SELECT email FROM account_users WHERE id = $1"
        )
            .bind(agent_id)
            .fetch_optional(pool(db))
            .await
            .unwrap_or(None);

        if let Some(agent_email_str) = agent_email {
            let _ = email_service.send_tour_assigned(
                &agent_email_str,
                &agent_name,
                &property_title,
                property_location.as_deref(),                 // Option<String> -> Option<&str>
                client_name.as_deref().unwrap_or("Client"),   // Option<String> -> &str
                &sla_deadline.to_string(),
            ).await.map_err(|e| tracing::warn!("⚠️ Failed to send tour assigned email: {}", e));
        }
    }

    Ok(serde_json::json!({
        "message": "Payment confirmed. Agent will fulfill tour within 24 hours.",
        "status": "pending"
    }))
}
// ───────────────────────────────────────────
// 3. Agent Uploads Native-Recorded Video
// ✅ MILESTONE 6: Auto-generates viewing link + sends email to client
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 3. Agent Uploads Native-Recorded Video
// ✅ MILESTONE 6: Auto-generates viewing link + sends email to client
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 3. Agent Uploads Native-Recorded Video
// ✅ MILESTONE 7: Wallet credit + Email + Auto viewing link
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 3. Agent Uploads Native-Recorded Video
// ✅ FIXED: Now credits KES 20 tour fee to agent wallet
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 3. Agent Uploads Native-Recorded Video
// ✅ MILESTONE 7: Wallet credit + Email + Auto viewing link
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 3. Agent Uploads Native-Recorded Video
// ✅ MILESTONE 6: Auto viewing link + email client
// ✅ MILESTONE 7: Credit KES 20 tour fee to agent wallet
// ───────────────────────────────────────────
pub async fn upload_tour_video(
    db: &rento_core::Database,
    email_service: &rento_core::email::EmailService,
    viewing_link_base: &str,
    agent_id: &Uuid,
    req: &UploadTourVideoRequest,
) -> ApiResult<serde_json::Value> {
    let tour_id = Uuid::parse_str(&req.tour_request_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid tour request ID: {}", e)))?;

    // Verify tour request exists and is assigned to this agent
    let tour_info: Option<(Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT property_id, assigned_agent_id, status FROM virtual_tour_requests WHERE id = $1"
    )
        .bind(tour_id)
        .fetch_optional(pool(db))
        .await?;

    let (property_id, assigned_agent, status) = match tour_info {
        Some(t) => t,
        None => return Err(ApiError::NotFound("Tour request not found".into())),
    };

    if assigned_agent != *agent_id {
        return Err(ApiError::Unauthorized("This tour is not assigned to you".into()));
    }

    if status != "pending" {
        return Err(ApiError::BadRequest(format!("Tour is already {}", status)));
    }

    // Parse recording timestamps
    let recording_started = req.recording_started_at.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));
    let recording_completed = req.recording_completed_at.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    // Create video record with watermark metadata
    let video_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO virtual_tour_videos
            (tour_request_id, property_id, agent_id, video_url, thumbnail_url,
             duration_seconds, file_size_bytes, watermark_agent_id, watermark_timestamp,
             device_fingerprint, recording_started_at, recording_completed_at, is_verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9, $10, $11, TRUE)
        RETURNING id
        "#
    )
        .bind(tour_id)
        .bind(property_id)
        .bind(agent_id)
        .bind(&req.video_url)
        .bind(&req.thumbnail_url)
        .bind(req.duration_seconds)
        .bind(req.file_size_bytes)
        .bind(agent_id.to_string())
        .bind(&req.device_fingerprint)
        .bind(recording_started)
        .bind(recording_completed)
        .fetch_one(pool(db))
        .await?;

    // Mark tour as fulfilled
    sqlx::query(
        "UPDATE virtual_tour_requests SET status = 'fulfilled', fulfilled_at = NOW(), updated_at = NOW() WHERE id = $1"
    )
        .bind(tour_id)
        .execute(pool(db))
        .await?;

    // Update agent SLA metrics
    update_agent_sla_on_fulfill(db, agent_id, tour_id).await?;

    // ═══════════════════════════════════════════
    // ✅ MILESTONE 7: CREDIT KES 20 TOUR FEE TO AGENT WALLET
    // ═══════════════════════════════════════════
    let mut wallet_credited = false;
    let mut new_balance = 0.0_f64;

    match crate::services::wallet::get_or_create_wallet(pool(db), agent_id).await {
        Ok(_) => {
            let mut tx = pool(db).begin().await?;

            // Create ledger entry for the tour fee
            let ledger_id: Uuid = sqlx::query_scalar(
                r#"
                INSERT INTO commission_ledger
                    (agent_id, payment_id, property_owner_id, property_id,
                     commission_type, gross_amount, commission_rate, commission_amount,
                     status, credited_at)
                VALUES ($1, NULL, NULL, $2, 'tour_fee', $3, 100.0, $3, 'credited', NOW())
                RETURNING id
                "#
            )
                .bind(agent_id)
                .bind(property_id)
                .bind(TOUR_FEE_KES)
                .fetch_one(&mut *tx)
                .await?;

            // Credit the wallet (updates balance AND total_earned)
            crate::services::wallet::credit_wallet(
                &mut tx,
                agent_id,
                TOUR_FEE_KES,
                &ledger_id.to_string(),
                &format!("🎬 Tour fee KES {:.2} for fulfilling tour {}", TOUR_FEE_KES, &tour_id.to_string()[..8]),
            ).await?;

            tx.commit().await?;
            wallet_credited = true;

            // Get updated balance
            if let Ok(wallet) = crate::services::wallet::get_or_create_wallet(pool(db), agent_id).await {
                new_balance = wallet.balance;
            }

            // Send commission email to agent
            let agent_email: Option<String> = sqlx::query_scalar(
                "SELECT email FROM account_users WHERE id = $1"
            )
                .bind(agent_id)
                .fetch_optional(pool(db))
                .await
                .unwrap_or(None);

            if let Some(email) = agent_email {
                let _ = email_service.send_commission_notification(
                    &email,
                    TOUR_FEE_KES,
                    TOUR_FEE_KES,
                    "tour_fee",
                ).await.map_err(|e| tracing::warn!("Failed to send tour fee email to agent: {}", e));
            }

            tracing::info!(
                "💰 Agent {} credited KES {:.2} for tour fulfillment (new balance: KES {:.2})",
                agent_id, TOUR_FEE_KES, new_balance
            );
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to credit tour fee to agent wallet: {}", e);
        }
    }

    // ═══════════════════════════════════════════
    // ✅ MILESTONE 6: AUTO-GENERATE VIEWING LINK + EMAIL CLIENT
    // ═══════════════════════════════════════════
    let tour_details: Option<(String, Option<String>, String, String)> = sqlx::query_as(
        r#"
        SELECT tr.client_email, tr.client_name, p.title, v.video_url
        FROM virtual_tour_requests tr
        JOIN properties p ON tr.property_id = p.id
        JOIN virtual_tour_videos v ON v.tour_request_id = tr.id
        WHERE tr.id = $1
        ORDER BY v.created_at DESC
        LIMIT 1
        "#
    )
        .bind(tour_id)
        .fetch_optional(pool(db))
        .await?;

    let mut generated_viewing_url: Option<String> = None;

    if let Some((client_email, client_name, property_title, _video_url)) = tour_details {
        // Generate viewing link automatically
        let viewing_result = generate_viewing_link(db, &tour_id.to_string(), None).await;

        if let Ok(viewing_data) = viewing_result {
            let viewing_url = viewing_data.get("viewing_url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let expires_at = viewing_data.get("expires_at")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            generated_viewing_url = Some(format!("{}{}", viewing_link_base, viewing_url));

            // Send "Tour Ready" email to client
            let _ = email_service.send_tour_fulfilled(
                &client_email,
                client_name.as_deref(),
                &property_title,
                viewing_url,
                expires_at,
                viewing_link_base,
            ).await.map_err(|e| tracing::warn!("⚠️ Failed to send tour fulfilled email to {}: {}", client_email, e));

            tracing::info!("📧 Auto-generated viewing link and emailed client {}", client_email);
        } else {
            tracing::warn!("⚠️ Failed to auto-generate viewing link for tour {}", tour_id);
        }
    }

    tracing::info!(
        "🎬 Agent {} uploaded tour video {} for request {} (watermarked)",
        agent_id, video_id, tour_id
    );

    Ok(serde_json::json!({
        "video_id": video_id.to_string(),
        "message": "Tour video uploaded successfully with watermark",
        "watermark": {
            "agent_id": agent_id.to_string(),
            "timestamp": chrono::Utc::now().to_string(),
            "logo": "R3NTO",
        },
        "client_notified": generated_viewing_url.is_some(),
        "wallet": {
            "credited": wallet_credited,
            "amount": TOUR_FEE_KES,
            "new_balance": new_balance,
            "can_request_payout": new_balance >= MINIMUM_PAYOUT_AMOUNT,
            "minimum_payout": MINIMUM_PAYOUT_AMOUNT,
        }
    }))
}
// ───────────────────────────────────────────
// 4. Generate Secure Viewing Link (2-hour + device lock)
// (UNCHANGED - still used for manual share button)
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 4. Generate Secure Viewing Link (7-day claim window + 2-hour viewing window)
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 4. Generate Secure Viewing Link (7-day claim + 2-hour viewing)
// ✅ FIXED: Reuses existing session instead of creating a new one
// ───────────────────────────────────────────
pub async fn generate_viewing_link(
    db: &rento_core::Database,
    request_id: &str,
    client_id: Option<Uuid>,
) -> ApiResult<serde_json::Value> {
    let req_id = Uuid::parse_str(request_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid request ID: {}", e)))?;

    // 1. Get tour request + video
    let tour_info: Option<(Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT v.id, v.video_url, tr.status
        FROM virtual_tour_requests tr
        JOIN virtual_tour_videos v ON v.tour_request_id = tr.id
        WHERE tr.id = $1
        ORDER BY v.created_at DESC
        LIMIT 1
        "#
    )
        .bind(req_id)
        .fetch_optional(pool(db))
        .await
        .map_err(|e| {
            tracing::error!("❌ DB ERROR fetching tour info: {:?}", e);
            ApiError::Internal(format!("Database error: {}", e))
        })?;

    let (video_id, video_url, status) = match tour_info {
        Some(t) => t,
        None => {
            tracing::warn!("⚠️ Tour not found or no video uploaded for request {}", req_id);
            return Err(ApiError::NotFound("Tour not found or video not uploaded yet".into()));
        }
    };

    if status != "fulfilled" {
        return Err(ApiError::BadRequest(format!("Tour status is '{}', must be 'fulfilled'", status)));
    }

    let now = chrono::Utc::now();

    // ✅ CHECK: Does an existing viewing session already exist for this tour?
    let existing_session: Option<(Uuid, String, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        r#"
        SELECT id, viewing_token, viewing_started_at, viewing_expires_at
        FROM tour_viewing_sessions
        WHERE tour_request_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
        .bind(req_id)
        .fetch_optional(pool(db))
        .await?;

    if let Some((session_id, existing_token, started_at, expires_at)) = existing_session {
        if let Some(started) = started_at {
            // ✅ Viewing window has already started (client clicked the link before)
            if let Some(expires) = expires_at {
                if now < expires {
                    // Still within the 2-hour window → REUSE existing session
                    tracing::info!("♻️ Reusing existing viewing session {} for tour {}", session_id, req_id);
                    return Ok(serde_json::json!({
                        "session_id": session_id.to_string(),
                        "viewing_token": existing_token,
                        "viewing_url": format!("/tour/view/{}", existing_token),
                        "video_url": video_url,
                        "window_minutes": VIEWING_WINDOW_MINUTES,
                    }));
                } else {
                    // ✅ 2-hour window has expired → reject
                    return Err(ApiError::BadRequest(
                        "The 2-hour viewing window for this tour has already expired.".into()
                    ));
                }
            }
        } else {
            // ✅ Not yet claimed — check if the 7-day claim window is still valid
            if let Some(claim_deadline) = expires_at {
                if now < claim_deadline {
                    // Still within claim window → REUSE existing unclaimed session
                    tracing::info!("♻️ Reusing unclaimed viewing session {} for tour {}", session_id, req_id);
                    return Ok(serde_json::json!({
                        "session_id": session_id.to_string(),
                        "viewing_token": existing_token,
                        "viewing_url": format!("/tour/view/{}", existing_token),
                        "video_url": video_url,
                        "window_minutes": VIEWING_WINDOW_MINUTES,
                    }));
                }
                // Claim window expired → fall through to create a new session below
            }
        }
    }

    // ✅ No valid existing session → create a NEW one
    let viewing_token = format!("vt_{}", Uuid::new_v4().to_string().replace("-", ""));
    let claim_deadline = chrono::Utc::now() + chrono::Duration::days(7);

    let session_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO tour_viewing_sessions
            (tour_request_id, video_id, client_id, viewing_token, viewing_expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
    )
        .bind(req_id)
        .bind(video_id)
        .bind(client_id)
        .bind(&viewing_token)
        .bind(claim_deadline)
        .fetch_one(pool(db))
        .await
        .map_err(|e| {
            tracing::error!("❌ DB ERROR inserting viewing session: {:?}", e);
            ApiError::Internal(format!("Database error: {}", e))
        })?;

    tracing::info!("✅ Generated NEW viewing token {} for tour {} (7-day claim window)", viewing_token, req_id);

    Ok(serde_json::json!({
        "session_id": session_id.to_string(),
        "viewing_token": viewing_token,
        "viewing_url": format!("/tour/view/{}", viewing_token),
        "video_url": video_url,
        "window_minutes": VIEWING_WINDOW_MINUTES,
    }))
}
// ───────────────────────────────────────────
// 5. Access Tour Video (validates 2-hour + device lock)
// ───────────────────────────────────────────
pub async fn access_tour_video(
    db: &rento_core::Database,
    viewing_token: &str,
    device_fingerprint: &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> ApiResult<serde_json::Value> {
    let session: Option<(Uuid, Uuid, Uuid, String, bool, Option<String>,
                         Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as(
            r#"
            SELECT id, tour_request_id, video_id, viewing_token, device_locked,
                   device_fingerprint, viewing_started_at, viewing_expires_at
            FROM tour_viewing_sessions WHERE viewing_token = $1
            "#
        )
            .bind(viewing_token)
            .fetch_optional(pool(db))
            .await?;

    let (session_id, _tour_request_id, video_id, _token, device_locked,
        locked_fingerprint, viewing_started, viewing_expires) = match session {
        Some(s) => s,
        None => return Err(ApiError::NotFound("Invalid viewing link".into())),
    };

    let now = chrono::Utc::now();

    // Check 2-hour expiry
    if viewing_started.is_some() {
        if let Some(expires) = viewing_expires {
            if now > expires {
                return Err(ApiError::BadRequest(
                    "Viewing link has expired. The 2-hour viewing window has ended.".into()
                ));
            }
        }
    }

    // Device locking logic
    if device_locked {
        if let Some(locked_fp) = locked_fingerprint {
            if locked_fp != device_fingerprint {
                tracing::warn!("🚫 Unauthorized device access attempt on tour session {}", session_id);
                return Err(ApiError::Unauthorized(
                    "This viewing link is locked to a different device".into()
                ));
            }
        }
    } else {
        // First access → lock to this device + start 2-hour window
        sqlx::query(
            r#"
            UPDATE tour_viewing_sessions
            SET device_locked = TRUE,
                device_fingerprint = $1,
                locked_at = NOW(),
                viewing_started_at = NOW(),
                viewing_expires_at = NOW() + INTERVAL '120 minutes'
            WHERE id = $2
            "#
        )
            .bind(device_fingerprint)
            .bind(session_id)
            .execute(pool(db))
            .await?;
    }

    // Update access count
    sqlx::query(
        r#"
        UPDATE tour_viewing_sessions
        SET access_count = access_count + 1,
            last_accessed_at = NOW(),
            ip_address = COALESCE($1, ip_address),
            user_agent = COALESCE($2, user_agent)
        WHERE id = $3
        "#
    )
        .bind(ip_address)
        .bind(user_agent)
        .bind(session_id)
        .execute(pool(db))
        .await?;

    // Get video URL
    let video_url: String = sqlx::query_scalar(
        "SELECT video_url FROM virtual_tour_videos WHERE id = $1"
    )
        .bind(video_id)
        .fetch_one(pool(db))
        .await?;

    // ✅ FETCH THE EXACT EXPIRY TIMESTAMP
    let actual_expires: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT viewing_expires_at FROM tour_viewing_sessions WHERE id = $1"
    )
        .bind(session_id)
        .fetch_one(pool(db))
        .await?;

    let remaining_minutes = actual_expires
        .map(|e| (e - now).num_minutes().max(0))
        .unwrap_or(VIEWING_WINDOW_MINUTES);

    // ✅ Convert to ISO 8601 string for the frontend
    let expires_at_iso = actual_expires
        .map(|e| e.to_rfc3339())
        .unwrap_or_default();

    Ok(serde_json::json!({
        "video_url": video_url,
        "session_id": session_id.to_string(),
        "device_locked": true,
        "remaining_minutes": remaining_minutes,
        "expires_at": expires_at_iso, // ✅ REQUIRED for frontend timer
    }))
}
// ───────────────────────────────────────────
// 6. Agent De-lists Property
// ───────────────────────────────────────────
pub async fn delist_property(
    db: &rento_core::Database,
    agent_id: &Uuid,
    property_id: &str,
    reason: Option<&str>,
) -> ApiResult<serde_json::Value> {
    let prop_uuid = Uuid::parse_str(property_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid property ID: {}", e)))?;

    let has_access: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM agent_conversions ac
            JOIN properties p ON p.owner_id = ac.property_owner_id
            WHERE p.id = $1 AND ac.agent_id = $2
        )
        "#
    )
        .bind(prop_uuid)
        .bind(agent_id)
        .fetch_one(pool(db))
        .await?;

    if !has_access {
        return Err(ApiError::Unauthorized("You don't have access to this property".into()));
    }

    sqlx::query(
        r#"
        UPDATE properties
        SET is_delisted = TRUE,
            delisted_at = NOW(),
            delisted_reason = $1,
            updated_at = NOW()
        WHERE id = $2
        "#
    )
        .bind(reason)
        .bind(prop_uuid)
        .execute(pool(db))
        .await?;

    let cancelled: u64 = sqlx::query(
        "UPDATE virtual_tour_requests SET status = 'property_delisted', updated_at = NOW() WHERE property_id = $1 AND status = 'pending'"
    )
        .bind(prop_uuid)
        .execute(pool(db))
        .await?
        .rows_affected();

    tracing::info!(
        "🚫 Agent {} de-listed property {} ({} pending tours cancelled)",
        agent_id, property_id, cancelled
    );

    Ok(serde_json::json!({
        "message": "Property de-listed successfully",
        "cancelled_tours": cancelled,
    }))
}

// ───────────────────────────────────────────
// 7. Agent's Pending Tours Dashboard (SLA tracking)
// ───────────────────────────────────────────
pub async fn get_agent_pending_tours(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            tr.id::text,
            tr.client_name,
            tr.client_email,
            tr.status,
            tr.sla_deadline,
            tr.created_at,
            tr.fee_paid,
            p.id::text as property_id,
            p.title as property_title,
            p.location as property_location,
            EXTRACT(EPOCH FROM (tr.sla_deadline - NOW()))::int as seconds_remaining
        FROM virtual_tour_requests tr
        JOIN properties p ON tr.property_id = p.id
        WHERE tr.assigned_agent_id = $1
          AND tr.status = 'pending'
          AND tr.fee_paid = TRUE
        ORDER BY tr.sla_deadline ASC
        "#
    )
        .bind(agent_id)
        .fetch_all(pool(db))
        .await?;

    let tours: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        let seconds_remaining: i32 = row.try_get("seconds_remaining").unwrap_or(0);
        let urgency = if seconds_remaining < 3600 {
            "critical"
        } else if seconds_remaining < 7200 {
            "urgent"
        } else if seconds_remaining < 43200 {
            "normal"
        } else {
            "plenty"
        };

        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "client_name": row.try_get::<Option<String>, _>("client_name").ok().flatten(),
            "client_email": row.try_get::<String, _>("client_email").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "sla_deadline": row.try_get::<chrono::DateTime<chrono::Utc>, _>("sla_deadline")
                .map(|d| d.to_string()).unwrap_or_default(),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
            "property_id": row.try_get::<String, _>("property_id").unwrap_or_default(),
            "property_title": row.try_get::<String, _>("property_title").unwrap_or_default(),
            "property_location": row.try_get::<Option<String>, _>("property_location").ok().flatten(),
            "seconds_remaining": seconds_remaining,
            "urgency": urgency,
        })
    }).collect();

    Ok(tours)
}

// ───────────────────────────────────────────
// 8. Agent Tour History (all statuses)
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// 8. Agent Tour History (all statuses)
// ✅ NEW: Includes viewing-link expiry status
// ───────────────────────────────────────────
pub async fn get_agent_tour_history(
    db: &rento_core::Database,
    agent_id: &Uuid,
    status_filter: Option<&str>,
    limit: Option<i64>,
) -> ApiResult<Vec<serde_json::Value>> {
    let mut query = String::from(
        r#"
        SELECT
            tr.id::text,
            tr.client_name,
            tr.client_email,
            tr.status,
            tr.fee_amount::TEXT as fee_amount,
            tr.created_at,
            tr.fulfilled_at,
            tr.sla_deadline,
            p.title as property_title,
            p.location as property_location,
            v.video_url,
            v.duration_seconds,
            CASE
                WHEN tr.fulfilled_at IS NOT NULL AND tr.fulfilled_at <= tr.sla_deadline THEN TRUE
                WHEN tr.fulfilled_at IS NOT NULL THEN FALSE
                ELSE NULL
            END as met_sla,
            -- ✅ NEW: Viewing session info (latest session per tour)
            vs.viewing_started_at,
            vs.viewing_expires_at,
            CASE
                WHEN vs.id IS NULL THEN 'not_generated'
                WHEN vs.viewing_expires_at <= NOW() THEN 'expired'
                WHEN vs.viewing_started_at IS NULL THEN 'awaiting_client'
                ELSE 'active'
            END as viewing_status
        FROM virtual_tour_requests tr
        JOIN properties p ON tr.property_id = p.id
        LEFT JOIN virtual_tour_videos v ON v.tour_request_id = tr.id
        LEFT JOIN LATERAL (
            SELECT id, viewing_started_at, viewing_expires_at
            FROM tour_viewing_sessions
            WHERE tour_request_id = tr.id
            ORDER BY created_at DESC
            LIMIT 1
        ) vs ON true
        WHERE tr.assigned_agent_id = $1
        "#
    );

    let mut param_idx = 2;
    if status_filter.is_some() {
        query.push_str(&format!(" AND tr.status = ${}", param_idx));
        param_idx += 1;
    }

    query.push_str(" ORDER BY tr.created_at DESC");

    if let Some(lim) = limit {
        query.push_str(&format!(" LIMIT {}", lim));
    }

    let mut q = sqlx::query(&query).bind(agent_id);
    if let Some(status) = status_filter {
        q = q.bind(status);
    }

    let rows = q.fetch_all(pool(db)).await?;

    let history: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "client_name": row.try_get::<Option<String>, _>("client_name").ok().flatten(),
            "client_email": row.try_get::<String, _>("client_email").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "fee_amount": row.try_get::<String, _>("fee_amount").unwrap_or_else(|_| "20.00".to_string()),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_rfc3339()).unwrap_or_default(),
            "fulfilled_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("fulfilled_at")
                .ok().flatten().map(|d| d.to_rfc3339()),
            "property_title": row.try_get::<String, _>("property_title").unwrap_or_default(),
            "property_location": row.try_get::<Option<String>, _>("property_location").ok().flatten(),
            "video_url": row.try_get::<Option<String>, _>("video_url").ok().flatten(),
            "duration_seconds": row.try_get::<Option<i32>, _>("duration_seconds").ok().flatten(),
            "met_sla": row.try_get::<Option<bool>, _>("met_sla").ok().flatten(),
            // ✅ NEW: Viewing link fields
            "viewing_status": row.try_get::<String, _>("viewing_status")
                .unwrap_or_else(|_| "not_generated".to_string()),
            "viewing_expires_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("viewing_expires_at")
                .ok().flatten().map(|d| d.to_rfc3339()),
        })
    }).collect();

    Ok(history)
}

// ───────────────────────────────────────────
// 9. Agent SLA Performance Stats
// ───────────────────────────────────────────
pub async fn get_agent_sla_stats(
    db: &rento_core::Database,
    agent_id: &Uuid,
) -> ApiResult<serde_json::Value> {
    let metrics: Option<(i32, i32, i32, i32, i32)> = sqlx::query_as(
        r#"
        SELECT
            total_tours_assigned,
            tours_fulfilled_on_time,
            tours_fulfilled_late,
            tours_expired,
            average_fulfillment_minutes
        FROM agent_sla_metrics
        WHERE agent_id = $1
        "#
    )
        .bind(agent_id)
        .fetch_optional(pool(db))
        .await?;

    let (total, on_time, late, expired, avg_minutes) = metrics
        .unwrap_or((0, 0, 0, 0, 0));

    let revenue: Option<String> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(fee_amount), 0)::TEXT
        FROM virtual_tour_requests
        WHERE assigned_agent_id = $1 AND status = 'fulfilled'
        "#
    )
        .bind(agent_id)
        .fetch_optional(pool(db))
        .await?;

    let total_fulfilled = on_time + late;
    let on_time_rate = if total_fulfilled > 0 {
        (on_time as f64 / total_fulfilled as f64 * 100.0).round() as i32
    } else {
        0
    };

    Ok(serde_json::json!({
        "total_tours_assigned": total,
        "tours_fulfilled_on_time": on_time,
        "tours_fulfilled_late": late,
        "tours_expired": expired,
        "total_fulfilled": total_fulfilled,
        "average_fulfillment_minutes": avg_minutes,
        "on_time_rate_percent": on_time_rate,
        "total_revenue_kes": revenue.unwrap_or_else(|| "0.00".to_string()),
    }))
}

// ───────────────────────────────────────────
// Helper: Update agent SLA metrics
// ───────────────────────────────────────────
async fn update_agent_sla_on_fulfill(
    db: &rento_core::Database,
    agent_id: &Uuid,
    tour_id: Uuid,
) -> ApiResult<()> {
    let on_time: Option<bool> = sqlx::query_scalar(
        "SELECT sla_deadline <= NOW() FROM virtual_tour_requests WHERE id = $1"
    )
        .bind(tour_id)
        .fetch_optional(pool(db))
        .await?;

    if let Some(was_on_time) = on_time {
        sqlx::query(
            r#"
            INSERT INTO agent_sla_metrics (agent_id, total_tours_assigned, tours_fulfilled_on_time, tours_fulfilled_late)
            VALUES ($1, 1, $2, $3)
            ON CONFLICT (agent_id) DO UPDATE SET
                total_tours_assigned = agent_sla_metrics.total_tours_assigned + 1,
                tours_fulfilled_on_time = agent_sla_metrics.tours_fulfilled_on_time + $2,
                tours_fulfilled_late = agent_sla_metrics.tours_fulfilled_late + $3,
                last_updated = NOW()
            "#
        )
            .bind(agent_id)
            .bind(if was_on_time { 1 } else { 0 })
            .bind(if was_on_time { 0 } else { 1 })
            .execute(pool(db))
            .await?;
    }

    Ok(())
}

// ───────────────────────────────────────────
// 10. Validate + Stream Tour Video (secure)
// ───────────────────────────────────────────
pub async fn validate_tour_stream_access(
    db: &rento_core::Database,
    viewing_token: &str,
    device_fingerprint: &str,
) -> ApiResult<String> {
    // ✅ Added viewing_started_at to SELECT
    let session: Option<(Uuid, Uuid, bool, Option<String>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as(
            r#"
            SELECT vs.id, vs.video_id, vs.device_locked, vs.device_fingerprint,
                   vs.viewing_started_at, vs.viewing_expires_at
            FROM tour_viewing_sessions vs
            WHERE vs.viewing_token = $1
            "#
        )
            .bind(viewing_token)
            .fetch_optional(pool(db))
            .await?;

    let (session_id, video_id, device_locked, locked_fingerprint, viewing_started, viewing_expires) =
        match session {
            Some(s) => s,
            None => return Err(ApiError::NotFound("Invalid viewing link".into())),
        };

    let now = chrono::Utc::now();

    // ✅ Block stream if 120 minutes have passed
    if viewing_started.is_some() {
        if let Some(expires) = viewing_expires {
            if now > expires {
                return Err(ApiError::BadRequest(
                    "This viewing link has expired. The 2-hour window has ended.".into()
                ));
            }
        }
    }

    if device_locked {
        if let Some(locked_fp) = locked_fingerprint {
            if locked_fp != device_fingerprint {
                tracing::warn!("🚫 Wrong device attempt on session {}", session_id);
                return Err(ApiError::Unauthorized(
                    "This tour is locked to a different device. Links cannot be shared.".into()
                ));
            }
        }
    } else {
        sqlx::query(
            r#"
            UPDATE tour_viewing_sessions
            SET device_locked = TRUE,
                device_fingerprint = $1,
                locked_at = NOW(),
                viewing_started_at = NOW(),
                viewing_expires_at = NOW() + INTERVAL '120 minutes'
            WHERE id = $2
            "#
        )
            .bind(device_fingerprint)
            .bind(session_id)
            .execute(pool(db))
            .await?;
    }

    sqlx::query(
        "UPDATE tour_viewing_sessions SET access_count = access_count + 1, last_accessed_at = NOW() WHERE id = $1"
    )
        .bind(session_id)
        .execute(pool(db))
        .await?;

    let video_url: String = sqlx::query_scalar(
        "SELECT video_url FROM virtual_tour_videos WHERE id = $1"
    )
        .bind(video_id)
        .fetch_one(pool(db))
        .await?;

    let file_path = video_url.trim_start_matches('/');
    Ok(file_path.to_string())
}

// ═══════════════════════════════════════════
// PUBLIC PROPERTY LISTINGS (No Auth Required)
// ═══════════════════════════════════════════
pub async fn get_public_properties(db: &rento_core::Database) -> ApiResult<Vec<Property>> {
    let rows: Vec<PropertyDbRow> = sqlx::query_as(
        r#"
        SELECT
            p.id, p.title, COALESCE(p.price, 0)::float8 as price, p.status::text as status,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as owner_name,
            COALESCE(p.county || ', ' || p.location, p.location, p.county, '') as location,
            COALESCE(p.property_type::text, '') as property_type,
            0 as bedrooms, 0 as bathrooms, 0 as area_sqft,
            p.created_at
        FROM properties p
        JOIN account_users u ON p.owner_id = u.id
        WHERE p.status = 'available' AND COALESCE(p.is_delisted, FALSE) = FALSE
        ORDER BY p.created_at DESC
        "#
    ).fetch_all(pool(db)).await?;
    Ok(rows.into_iter().map(Property::from).collect())
}

pub async fn get_public_property_detail(db: &rento_core::Database, id: &str) -> ApiResult<PropertyDetail> {
    let property_id = Uuid::parse_str(id).map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;
    let row = sqlx::query(
        r#"
        SELECT
            p.id, p.title, p.description, COALESCE(p.price, 0)::float8 as price, p.status::text as status,
            COALESCE(p.county || ', ' || p.location, p.location, p.county, '') as location,
            COALESCE(p.property_type::text, '') as property_type,
            0 as bedrooms, 0 as bathrooms, 0 as area_sqft,
            '{}'::text[] as features, '{}'::text[] as images,
            p.created_at as listing_date, 0 as views, 0 as inquiries,
            u.id as owner_id,
            COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ' '), u.username) as owner_name,
            'hidden' as owner_email, 'PROPERTY_OWNER' as owner_role
        FROM properties p
        JOIN account_users u ON p.owner_id = u.id
        WHERE p.id = $1 AND p.status = 'available' AND COALESCE(p.is_delisted, FALSE) = FALSE
        "#
    ).bind(property_id).fetch_optional(pool(db)).await?;

    let row = row.ok_or_else(|| ApiError::NotFound("Property not found or not available".into()))?;

    let owner = PropertyOwner {
        id: row.try_get::<sqlx::types::Uuid, _>("owner_id")?.to_string(),
        name: row.try_get("owner_name")?,
        email: "Contact via platform".to_string(), // Hide email publicly
        role: row.try_get("owner_role")?,
    };

    Ok(PropertyDetail {
        id: row.try_get::<sqlx::types::Uuid, _>("id")?.to_string(),
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        price: row.try_get::<f64, _>("price")?,
        status: row.try_get("status")?,
        owner,
        location: row.try_get("location")?,
        property_type: row.try_get("property_type")?,
        bedrooms: row.try_get::<i32, _>("bedrooms")? as u32,
        bathrooms: row.try_get::<i32, _>("bathrooms")? as u32,
        area_sqft: row.try_get::<i32, _>("area_sqft")? as u32,
        features: row.try_get::<Vec<String>, _>("features").unwrap_or_default(),
        images: row.try_get::<Vec<String>, _>("images").unwrap_or_default(),
        listing_date: row.try_get::<chrono::DateTime<chrono::Utc>, _>("listing_date")?.format("%Y-%m-%d").to_string(),
        views: row.try_get::<i32, _>("views")? as u32,
        inquiries: row.try_get::<i32, _>("inquiries")? as u32,
    })
}

// Add at the bottom of services/admin.rs
pub async fn get_client_tours(
    db: &rento_core::Database,
    client_id: &Uuid,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            tr.id::text,
            tr.client_name,
            tr.status,
            tr.fee_amount::TEXT as fee_amount,
            tr.fee_paid,
            tr.created_at,
            tr.fulfilled_at,
            tr.sla_deadline,
            p.title as property_title,
            p.location as property_location,
            v.video_url,
            v.duration_seconds
        FROM virtual_tour_requests tr
        JOIN properties p ON tr.property_id = p.id
        LEFT JOIN virtual_tour_videos v ON v.tour_request_id = tr.id
        WHERE tr.client_id = $1
        ORDER BY tr.created_at DESC
        LIMIT 20
        "#
    )
        .bind(client_id)
        .fetch_all(pool(db))
        .await?;

    let tours: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "client_name": row.try_get::<Option<String>, _>("client_name").ok().flatten(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "fee_amount": row.try_get::<String, _>("fee_amount").unwrap_or_else(|_| "20.00".to_string()),
            "fee_paid": row.try_get::<bool, _>("fee_paid").unwrap_or(false),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default(),
            "fulfilled_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("fulfilled_at")
                .ok().flatten().map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
            "property_title": row.try_get::<String, _>("property_title").unwrap_or_default(),
            "property_location": row.try_get::<Option<String>, _>("property_location").ok().flatten(),
            "video_url": row.try_get::<Option<String>, _>("video_url").ok().flatten(),
            "duration_seconds": row.try_get::<Option<i32>, _>("duration_seconds").ok().flatten(),
        })
    }).collect();

    Ok(tours)
}

// ═══════════════════════════════════════════
// ADMIN TOUR OVERSIGHT — Full lifecycle visibility
// ═══════════════════════════════════════════
pub async fn get_all_tours_admin(
    db: &rento_core::Database,
    status_filter: Option<&str>,
    limit: Option<i64>,
) -> ApiResult<Vec<serde_json::Value>> {
    let mut query = String::from(
        r#"
        SELECT
            tr.id::text as tour_id,
            tr.client_email,
            tr.client_name,
            tr.client_phone,
            tr.status,
            tr.fee_amount::TEXT as fee_amount,
            tr.fee_paid,
            tr.payment_reference,
            tr.created_at as requested_at,
            tr.fulfilled_at,
            tr.sla_deadline,
            -- Property info
            p.id::text as property_id,
            p.title as property_title,
            p.location as property_location,
            -- Agent info
            COALESCE(NULLIF(ag.first_name || ' ' || ag.last_name, ' '), ag.username) as agent_name,
            ag.email as agent_email,
            -- Video info
            v.id::text as video_id,
            v.video_url,
            v.duration_seconds,
            v.created_at as video_uploaded_at,
            -- Viewing sessions
            (SELECT COUNT(*) FROM tour_viewing_sessions vs WHERE vs.tour_request_id = tr.id) as viewing_sessions_count,
            -- SLA calculation
            CASE
                WHEN tr.fulfilled_at IS NOT NULL AND tr.fulfilled_at <= tr.sla_deadline THEN 'on_time'
                WHEN tr.fulfilled_at IS NOT NULL THEN 'late'
                WHEN tr.status = 'expired' THEN 'expired'
                WHEN NOW() > tr.sla_deadline AND tr.status = 'pending' THEN 'overdue'
                ELSE 'within_sla'
            END as sla_status,
            -- Time to fulfill (in minutes)
            CASE
                WHEN tr.fulfilled_at IS NOT NULL THEN
                    EXTRACT(EPOCH FROM (tr.fulfilled_at - tr.created_at))::int / 60
                ELSE NULL
            END as fulfillment_minutes
        FROM virtual_tour_requests tr
        JOIN properties p ON tr.property_id = p.id
        LEFT JOIN account_users ag ON tr.assigned_agent_id = ag.id
        LEFT JOIN virtual_tour_videos v ON v.tour_request_id = tr.id
        WHERE 1=1
        "#
    );

    let mut param_idx = 1;
    if let Some(status) = status_filter {
        query.push_str(&format!(" AND tr.status = ${}", param_idx));
        param_idx += 1;
    }

    query.push_str(" ORDER BY tr.created_at DESC");

    if let Some(lim) = limit {
        query.push_str(&format!(" LIMIT {}", lim));
    }

    let mut q = sqlx::query(&query);
    if let Some(status) = status_filter {
        q = q.bind(status);
    }

    let rows = q.fetch_all(pool(db)).await?;

    let tours: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "tour_id": row.try_get::<String, _>("tour_id").unwrap_or_default(),
            "client_email": row.try_get::<String, _>("client_email").unwrap_or_default(),
            "client_name": row.try_get::<Option<String>, _>("client_name").ok().flatten(),
            "client_phone": row.try_get::<Option<String>, _>("client_phone").ok().flatten(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "fee_amount": row.try_get::<String, _>("fee_amount").unwrap_or_else(|_| "20.00".to_string()),
            "fee_paid": row.try_get::<bool, _>("fee_paid").unwrap_or(false),
            "payment_reference": row.try_get::<Option<String>, _>("payment_reference").ok().flatten(),
            "requested_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("requested_at")
                .map(|d| d.to_rfc3339()).unwrap_or_default(),
            "fulfilled_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("fulfilled_at")
                .ok().flatten().map(|d| d.to_rfc3339()),
            "sla_deadline": row.try_get::<chrono::DateTime<chrono::Utc>, _>("sla_deadline")
                .map(|d| d.to_rfc3339()).unwrap_or_default(),
            "property_id": row.try_get::<String, _>("property_id").unwrap_or_default(),
            "property_title": row.try_get::<String, _>("property_title").unwrap_or_default(),
            "property_location": row.try_get::<Option<String>, _>("property_location").ok().flatten(),
            "agent_name": row.try_get::<Option<String>, _>("agent_name").ok().flatten(),
            "agent_email": row.try_get::<Option<String>, _>("agent_email").ok().flatten(),
            "video_id": row.try_get::<Option<String>, _>("video_id").ok().flatten(),
            "video_url": row.try_get::<Option<String>, _>("video_url").ok().flatten(),
            "duration_seconds": row.try_get::<Option<i32>, _>("duration_seconds").ok().flatten(),
            "video_uploaded_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("video_uploaded_at")
                .ok().flatten().map(|d| d.to_rfc3339()),
            "viewing_sessions_count": row.try_get::<i64, _>("viewing_sessions_count").unwrap_or(0),
            "sla_status": row.try_get::<String, _>("sla_status").unwrap_or_default(),
            "fulfillment_minutes": row.try_get::<Option<i32>, _>("fulfillment_minutes").ok().flatten(),
        })
    }).collect();

    Ok(tours)
}

// Admin tour stats summary
pub async fn get_tour_stats_admin(
    db: &rento_core::Database,
) -> ApiResult<serde_json::Value> {
    let stats: (i64, i64, i64, i64, i64, Option<f64>) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'pending') as pending,
            COUNT(*) FILTER (WHERE status = 'fulfilled') as fulfilled,
            COUNT(*) FILTER (WHERE status = 'expired') as expired,
            COUNT(*) FILTER (WHERE status = 'property_delisted') as delisted,
            COALESCE(SUM(fee_amount) FILTER (WHERE status = 'fulfilled'), 0)::float8 as total_revenue
        FROM virtual_tour_requests
        "#
    )
        .fetch_one(pool(db))
        .await?;

    let avg_fulfillment: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (fulfilled_at - created_at)) / 3600.0)
        FROM virtual_tour_requests
        WHERE fulfilled_at IS NOT NULL
        "#
    )
        .fetch_optional(pool(db))
        .await?;

    Ok(serde_json::json!({
        "total_tours": stats.0,
        "pending": stats.1,
        "fulfilled": stats.2,
        "expired": stats.3,
        "delisted": stats.4,
        "total_revenue_kes": stats.5.unwrap_or(0.0),
        "avg_fulfillment_hours": avg_fulfillment.unwrap_or(0.0),
    }))
}
