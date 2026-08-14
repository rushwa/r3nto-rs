use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use rand::Rng;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use sqlx::Row;
use uuid::Uuid;

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
pub async fn update_user_role(db: &rento_core::Database, user_id: &str, role: &str, is_superuser: bool, is_staff: bool) -> ApiResult<()> {
    let id = Uuid::parse_str(user_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    // If making this user a superuser, check no other superuser exists
    if is_superuser {
        let existing_superuser: Option<(String,)> = sqlx::query_as(
            "SELECT id::text FROM account_users WHERE is_superuser = true AND id != $1 LIMIT 1"
        )
            .bind(id)
            .fetch_optional(pool(db)).await?;

        if existing_superuser.is_some() {
            return Err(ApiError::BadRequest("A superuser already exists. Demote the current superuser first.".to_string()));
        }
    }

    sqlx::query(
        "UPDATE account_users SET is_superuser = $1, is_staff = $2, role = $3::text::user_role, updated_at = NOW() WHERE id = $4"
    )
        .bind(is_superuser)
        .bind(is_staff)
        .bind(role)
        .bind(id)
        .execute(pool(db)).await?;
    Ok(())
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