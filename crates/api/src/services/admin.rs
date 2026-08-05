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



pub async fn initiate_handshake(db: &rento_core::Database, agent_id: &str, target_identifier: &str) -> ApiResult<()> {
    let target_uuid = if let Ok(uuid) = Uuid::parse_str(target_identifier) {
        uuid
    } else {
        let email_uuid: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM account_users WHERE email = $1"
        )
            .bind(target_identifier)
            .fetch_optional(pool(db))
            .await?;

        match email_uuid {
            Some(id) => id,
            None => return Err(ApiError::NotFound("User not found with this email or UUID".to_string())),
        }
    };

    let (email, current_role): (String, String) = sqlx::query_as(
        "SELECT email, role::text FROM account_users WHERE id = $1"
    )
        .bind(target_uuid)
        .fetch_one(pool(db))
        .await
        .map_err(|_| ApiError::NotFound("User not found".to_string()))?;

    if current_role != "CLIENT" {
        return Err(ApiError::BadRequest(format!(
            "Target user is currently a '{}' and cannot be converted.",
            current_role
        )));
    }

    let otp = format!("{:06}", rand::thread_rng().gen_range(0..1000000));
    let expires_at = Utc::now() + Duration::minutes(15);

    // FIX: Use UPSERT to replace any existing OTP for this email
    sqlx::query(
        "INSERT INTO email_otps (email, code, purpose, expires_at, is_used)
         VALUES ($1, $2, 'ROLE_CONVERSION', $3, false)
         ON CONFLICT (email) DO UPDATE
         SET code = $2, purpose = 'ROLE_CONVERSION', expires_at = $3, is_used = false"
    )
        .bind(&email)
        .bind(&otp)
        .bind(expires_at)
        .execute(pool(db))
        .await?;

    tracing::info!("🛡️ HANDSHAKE OTP for user {} ({}): {} (Simulated email send)", target_identifier, email, otp);

    Ok(())
}
pub async fn verify_handshake(db: &rento_core::Database, agent_id: &str, target_identifier: &str, otp_code: &str) -> ApiResult<()> {
    let agent_uuid = Uuid::parse_str(agent_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid Agent UUID: {}", e)))?;

    // 1. Try to parse as UUID first. If it fails, assume it's an email and look it up.
    let target_uuid = if let Ok(uuid) = Uuid::parse_str(target_identifier) {
        uuid
    } else {
        let email_uuid: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM account_users WHERE email = $1"
        )
            .bind(target_identifier)
            .fetch_optional(pool(db))
            .await?;

        match email_uuid {
            Some(id) => id,
            None => return Err(ApiError::NotFound("User not found with this email or UUID".to_string())),
        }
    };

    // 2. Get user's email to check OTP
    let email: String = sqlx::query_scalar("SELECT email FROM account_users WHERE id = $1")
        .bind(target_uuid)
        .fetch_one(pool(db))
        .await
        .map_err(|_| ApiError::NotFound("User not found".to_string()))?;

    // 3. Verify OTP (get the most recent unused ROLE_CONVERSION OTP for this email)
    let otp_record: Option<(Uuid, bool, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, is_used, expires_at FROM email_otps
         WHERE email = $1 AND code = $2 AND purpose = 'ROLE_CONVERSION'
         ORDER BY created_at DESC LIMIT 1"
    )
        .bind(&email)
        .bind(otp_code)
        .fetch_optional(pool(db))
        .await?;

    let (otp_id, is_used, expires_at) = match otp_record {
        Some(record) => record,
        None => return Err(ApiError::Unauthorized("Invalid OTP code".to_string())),
    };

    if is_used {
        return Err(ApiError::BadRequest("This OTP has already been used".to_string()));
    }
    if Utc::now() > expires_at {
        return Err(ApiError::BadRequest("This OTP has expired".to_string()));
    }

    // 4. Mark OTP as used
    sqlx::query("UPDATE email_otps SET is_used = true WHERE id = $1")
        .bind(otp_id)
        .execute(pool(db))
        .await?;

    // 5. Promote user to PROPERTY_OWNER
    sqlx::query("UPDATE account_users SET role = 'PROPERTY_OWNER', updated_at = NOW() WHERE id = $1")
        .bind(target_uuid)
        .execute(pool(db))
        .await?;

    // 6. Record the conversion relationship (enforces "only see owners they converted")
    sqlx::query(
        "INSERT INTO agent_conversions (agent_id, property_owner_id, converted_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (property_owner_id) DO NOTHING"
    )
        .bind(agent_uuid)
        .bind(target_uuid)
        .execute(pool(db))
        .await?;

    Ok(())
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
pub async fn get_agent_leads(db: &rento_core::Database, claims: &Claims) -> ApiResult<Vec<serde_json::Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::BadRequest(format!("Invalid UUID: {}", e)))?;

    let is_agent = claims.role.to_uppercase() == "AGENT";

    let rows = if is_agent {
        // AGENT: Only see leads they have claimed
        sqlx::query(
            r#"
            SELECT
                id::text, email, full_name, phone, status::text,
                claimed_by::text, created_at, updated_at
            FROM agent_leads
            WHERE claimed_by = $1
            ORDER BY created_at DESC
            "#
        )
            .bind(user_id)
            .fetch_all(pool(db)).await?
    } else {
        // ADMIN/SUPERUSER: See all leads
        sqlx::query(
            r#"
            SELECT
                id::text, email, full_name, phone, status::text,
                claimed_by::text, created_at, updated_at
            FROM agent_leads
            ORDER BY created_at DESC
            "#
        )
            .fetch_all(pool(db)).await?
    };

    // Map the rows to JSON (matching your get_user_profile pattern)
    let leads: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "email": row.try_get::<String, _>("email").unwrap_or_default(),
            "full_name": row.try_get::<String, _>("full_name").unwrap_or_default(),
            "phone": row.try_get::<Option<String>, _>("phone").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "claimed_by": row.try_get::<Option<String>, _>("claimed_by").unwrap_or_default(),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").map(|d| d.to_string()).unwrap_or_default(),
            "updated_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").map(|d| d.to_string()).unwrap_or_default(),
        })
    }).collect();

    Ok(leads)
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

pub async fn get_subscription_plans(db: &rento_core::Database) -> ApiResult<Vec<SubscriptionPlan>> {
    let rows: Vec<SubscriptionPlanDbRow> = sqlx::query_as(
        r#"
        SELECT
            id, name, price::float8,
            COALESCE(features, '{}')::text[] as features,
            0 as subscribers
        FROM subscription_plans
        ORDER BY price
        "#
    )
        .fetch_all(pool(db)).await?;

    Ok(rows.into_iter().map(SubscriptionPlan::from).collect())
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
