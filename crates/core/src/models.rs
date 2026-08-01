// crates/core/src/models.rs
// Translation of Django models to Rust structs

use chrono::{DateTime, Utc, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ==================== USER MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Admin,
    Agent,
    PropertyOwner,
    Client,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Client
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "ADMIN"),
            UserRole::Agent => write!(f, "AGENT"),
            UserRole::PropertyOwner => write!(f, "PROPERTY_OWNER"),
            UserRole::Client => write!(f, "CLIENT"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct AccountUser {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    #[validate(length(min = 8))]
    pub password_hash: String,
    pub identification_no: Option<String>,
    pub phone_number: Option<String>,
    pub role: UserRole,
    pub first_name: String,
    pub last_name: String,
    pub profile: Option<String>,

    // Location fields
    pub county: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub location: Option<String>,

    // Phone verification
    pub phone_verified: bool,
    pub phone_verification_code: Option<String>,
    pub phone_verification_sent_at: Option<DateTime<Utc>>,

    pub is_staff: bool,
    pub is_active: bool,
    pub is_superuser: bool,
    pub date_joined: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub subscribed: bool,
    pub subscription_date: Option<DateTime<Utc>>,
}

impl AccountUser {
    pub fn get_full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name).trim().to_string()
    }

    pub fn get_short_name(&self) -> String {
        self.first_name.clone()
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin || self.is_superuser
    }

    pub fn is_agent(&self) -> bool {
        self.role == UserRole::Agent
    }

    pub fn is_property_owner(&self) -> bool {
        self.role == UserRole::PropertyOwner
    }

    pub fn is_client(&self) -> bool {
        self.role == UserRole::Client
    }
}

// ==================== PROFILE MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub agent_id: Uuid,
    pub total_commissions: Decimal,
    pub pending_commissions: Decimal,
    pub paid_commissions: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PropertyOwnerProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub properties_owned: i32,
    pub subscription_tier: String, // basic, premium, enterprise
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== COMMISSION MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "commission_status", rename_all = "UPPERCASE")]
pub enum CommissionStatus {
    Pending,
    Approved,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Commission {
    pub id: Uuid,
    pub agent_id: Uuid,        // AgentProfile.id
    pub property_owner_id: Uuid, // AccountUser.id
    pub amount: Decimal,
    pub commission_percentage: Decimal,
    pub status: CommissionStatus,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

// ==================== VERIFICATION MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "verification_purpose", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationPurpose {
    Registration,
    PasswordReset,
    PhoneUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PhoneVerification {
    pub id: Uuid,
    pub phone_number: String,
    pub email: String,
    pub verification_code: String,
    pub purpose: VerificationPurpose,
    pub is_verified: bool,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl PhoneVerification {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn can_be_used(&self) -> bool {
        !self.is_expired() && !self.is_used && self.is_verified
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailOtp {
    pub id: Uuid,
    pub email: String,
    pub code: String,
    pub purpose: VerificationPurpose,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl EmailOtp {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WhatsAppOtp {
    pub id: Uuid,
    pub phone_number: String,
    pub code: String,
    pub purpose: VerificationPurpose,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// ==================== PROPERTY MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "property_type", rename_all = "lowercase")]
pub enum PropertyType {
    Apartment,
    House,
    Commercial,
    Land,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "property_status", rename_all = "lowercase")]
pub enum PropertyStatus {
    Available,
    Occupied,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "subscription_status", rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Inactive,
    Expired,
    Cancelled,
    Trial,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PropertyInformation {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub property_type: Option<PropertyType>,
    pub price: Option<Decimal>,
    pub subscription_status: SubscriptionStatus,
    pub subscription_tier: Option<String>,
    pub subscription_start_date: Option<DateTime<Utc>>,
    pub subscription_end_date: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub owner_id: Uuid,
    pub status: PropertyStatus,
    pub county: Option<String>,
    pub location: Option<String>,
    pub plot_number: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub purpose: Option<String>,
    pub general_features: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "unit_type", rename_all = "lowercase")]
pub enum UnitType {
    Single,
    Double,
    Bedsitter,
    #[sqlx(rename = "1bed")]
    OneBed,
    #[sqlx(rename = "2bed")]
    TwoBed,
    #[sqlx(rename = "3bed")]
    ThreeBed,
    Apartment,
    Bungalow,
    Villa,
    Land,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "unit_status", rename_all = "lowercase")]
pub enum UnitStatus {
    Vacant,
    Occupied,
    ToLet,
    Available,
    Sold,
    UnderOffer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "unit_purpose", rename_all = "lowercase")]
pub enum UnitPurpose {
    Rent,
    Sale,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PropertyTypeInformation {
    pub id: Uuid,
    pub property_id: Uuid,
    pub property_type: Option<UnitType>,
    pub price: Option<Decimal>,
    pub status: UnitStatus,
    pub purpose: UnitPurpose,
    pub description: Option<String>,
    pub is_active: bool,
    pub floor: Option<i32>,
    pub unit_number: Option<String>,
    pub size_sqft: Option<Decimal>,
    pub specific_features: Option<serde_json::Value>,
    pub total_units: Option<i32>,
    pub vacant_units: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PropertyImage {
    pub id: Uuid,
    pub property_id: Uuid,
    pub image_url: String,
    pub caption: Option<String>,
    pub is_main: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UnitImage {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub image_url: String,
    pub caption: Option<String>,
    pub is_main: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Amenity {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PropertyAmenity {
    pub id: Uuid,
    pub property_id: Uuid,
    pub amenity_id: Uuid,
    pub is_available: bool,
}

// ==================== SUBSCRIPTION MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "plan_tier", rename_all = "lowercase")]
pub enum PlanTier {
    FreeTier,
    FreeTrial,
    Basic,
    Professional,
    Premium,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "plan_duration", rename_all = "lowercase")]
pub enum PlanDuration {
    Trial,
    Monthly,
    Quarterly,
    Yearly,
    Permanent,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub tier: PlanTier,
    pub price: Decimal,
    pub duration: PlanDuration,
    pub properties_limit: i32,
    pub features: Option<serde_json::Value>,
    pub max_images_per_property: i32,
    pub max_units_per_property: i32,
    pub analytics_access: bool,
    pub priority_support: bool,
    pub featured_listing: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PropertySubscription {
    pub id: Uuid,
    pub property_id: Uuid,
    pub plan_id: Uuid,
    pub status: SubscriptionStatus,
    pub amount_paid: Decimal,
    pub transaction_id: Option<String>,
    pub payment_method: Option<String>,
    pub payment_status: PaymentStatus,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PropertySubscription {
    pub fn is_currently_active(&self) -> bool {
        let now = Utc::now();
        matches!(self.status, SubscriptionStatus::Active | SubscriptionStatus::Trial)
            && self.start_date <= now
            && now <= self.end_date
    }

    pub fn is_trial(&self) -> bool {
        self.status == SubscriptionStatus::Trial
    }

    pub fn days_remaining(&self) -> i64 {
        let now = Utc::now();
        if now > self.end_date {
            0
        } else {
            (self.end_date - now).num_days()
        }
    }
}

// ==================== AUTH MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: Uuid,      // user id
    pub role: UserRole,
    pub username: String,
    pub email: String,
    pub exp: i64,       // expiration
    pub iat: i64,       // issued at
}

// ==================== REQUEST/RESPONSE DTOs ====================

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    pub phone_number: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub verification_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub phone_number: Option<String>,
    pub identification_no: Option<String>,
    pub county: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub location: Option<String>,
    pub phone_verified: bool,
    pub subscribed: bool,
    pub is_active: bool,
}

impl From<AccountUser> for UserResponse {
    fn from(user: AccountUser) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role,
            phone_number: user.phone_number,
            identification_no: user.identification_no,
            county: user.county,
            constituency: user.constituency,
            ward: user.ward,
            location: user.location,
            phone_verified: user.phone_verified,
            subscribed: user.subscribed,
            is_active: user.is_active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CompleteProfileRequest {
    #[validate(length(min = 1))]
    pub first_name: String,
    #[validate(length(min = 1))]
    pub last_name: String,
    #[validate(length(min = 1))]
    pub identification_no: String,
    #[validate(length(min = 1))]
    pub county: String,
    #[validate(length(min = 1))]
    pub constituency: String,
    #[validate(length(min = 1))]
    pub ward: String,
    #[validate(length(min = 1))]
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PropertyCreateRequest {
    #[validate(length(min = 1))]
    pub title: String,
    pub description: Option<String>,
    pub property_type: Option<PropertyType>,
    pub price: Option<Decimal>,
    pub county: Option<String>,
    pub location: Option<String>,
    pub plot_number: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub purpose: Option<String>,
    pub general_features: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub property_type: Option<String>,
    pub price: Option<Decimal>,
    pub owner: UserResponse,
    pub images: Vec<PropertyImage>,
    pub units: Vec<UnitResponse>,
    pub subscription_status: String,
    pub subscription_tier: Option<String>,
    pub is_active: bool,
    pub county: Option<String>,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitResponse {
    pub id: Uuid,
    pub property_id: Uuid,
    pub property_type: Option<String>,
    pub price: Option<Decimal>,
    pub status: String,
    pub purpose: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub images: Vec<UnitImage>,
    pub total_units: Option<i32>,
    pub vacant_units: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommissionResponse {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub property_owner_id: Uuid,
    pub property_owner_name: String,
    pub amount: Decimal,
    pub commission_percentage: Decimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatsResponse {
    pub total_agents: i64,
    pub active_agents: i64,
    pub recent_registrations_30_days: i64,
    pub total_commissions_all_agents: Decimal,
    pub total_pending_commissions: Decimal,
    pub total_property_owners_registered: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionPlanResponse {
    pub id: Uuid,
    pub name: String,
    pub tier: String,
    pub price: Decimal,
    pub duration: String,
    pub properties_limit: i32,
    pub max_images_per_property: i32,
    pub max_units_per_property: i32,
    pub analytics_access: bool,
    pub priority_support: bool,
    pub featured_listing: bool,
    pub features: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub property_id: Uuid,
    pub property_title: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub plan_tier: String,
    pub plan_price: Decimal,
    pub status: String,
    pub amount_paid: Decimal,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub days_remaining: i64,
    pub is_trial: bool,
    pub is_currently_active: bool,
}
