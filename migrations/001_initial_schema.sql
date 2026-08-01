-- migrations/001_initial_schema.sql
-- RentoLink Database Schema (PostgreSQL)

-- Create custom types
CREATE TYPE user_role AS ENUM ('ADMIN', 'AGENT', 'PROPERTY_OWNER', 'CLIENT');
CREATE TYPE commission_status AS ENUM ('PENDING', 'APPROVED', 'PAID');
CREATE TYPE verification_purpose AS ENUM ('REGISTRATION', 'PASSWORD_RESET', 'PHONE_UPDATE');
CREATE TYPE property_type AS ENUM ('apartment', 'house', 'commercial', 'land');
CREATE TYPE property_status AS ENUM ('available', 'occupied', 'maintenance');
CREATE TYPE subscription_status AS ENUM ('pending', 'active', 'inactive', 'expired', 'cancelled', 'trial');
CREATE TYPE unit_type AS ENUM ('single', 'double', 'bedsitter', '1bed', '2bed', '3bed', 'apartment', 'bungalow', 'villa', 'land');
CREATE TYPE unit_status AS ENUM ('vacant', 'occupied', 'to_let', 'available', 'sold', 'under_offer');
CREATE TYPE unit_purpose AS ENUM ('rent', 'sale');
CREATE TYPE plan_tier AS ENUM ('free_tier', 'free_trial', 'basic', 'professional', 'premium');
CREATE TYPE plan_duration AS ENUM ('trial', 'monthly', 'quarterly', 'yearly', 'permanent');
CREATE TYPE payment_status AS ENUM ('pending', 'completed', 'failed', 'refunded');

-- Account Users table
CREATE TABLE account_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    identification_no VARCHAR(8) UNIQUE,
    phone_number VARCHAR(15),
    role user_role NOT NULL DEFAULT 'CLIENT',
    first_name VARCHAR(255) NOT NULL DEFAULT '',
    last_name VARCHAR(255) NOT NULL DEFAULT '',
    profile VARCHAR(255),
    county VARCHAR(100),
    constituency VARCHAR(100),
    ward VARCHAR(100),
    location VARCHAR(100),
    phone_verified BOOLEAN NOT NULL DEFAULT FALSE,
    phone_verification_code VARCHAR(6),
    phone_verification_sent_at TIMESTAMPTZ,
    is_staff BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    is_superuser BOOLEAN NOT NULL DEFAULT FALSE,
    date_joined TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    subscribed BOOLEAN NOT NULL DEFAULT FALSE,
    subscription_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_account_users_email ON account_users(email);
CREATE INDEX idx_account_users_role ON account_users(role);
CREATE INDEX idx_account_users_phone ON account_users(phone_number);

-- Agent Profiles
CREATE TABLE agent_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    agent_id UUID UNIQUE NOT NULL,
    total_commissions DECIMAL(10,2) NOT NULL DEFAULT 0,
    pending_commissions DECIMAL(10,2) NOT NULL DEFAULT 0,
    paid_commissions DECIMAL(10,2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_agent_profiles_user ON agent_profiles(user_id);

-- Property Owner Profiles
CREATE TABLE property_owner_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    properties_owned INTEGER NOT NULL DEFAULT 0,
    subscription_tier VARCHAR(20) NOT NULL DEFAULT 'basic',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_property_owner_profiles_user ON property_owner_profiles(user_id);

-- Commissions
CREATE TABLE commissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agent_profiles(id) ON DELETE CASCADE,
    property_owner_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    amount DECIMAL(10,2) NOT NULL DEFAULT 0,
    commission_percentage DECIMAL(5,2) NOT NULL DEFAULT 10,
    status commission_status NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at TIMESTAMPTZ,
    UNIQUE(agent_id, property_owner_id)
);

CREATE INDEX idx_commissions_agent ON commissions(agent_id);
CREATE INDEX idx_commissions_property_owner ON commissions(property_owner_id);

-- Phone Verifications
CREATE TABLE phone_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number VARCHAR(15) NOT NULL,
    email VARCHAR(255) NOT NULL,
    verification_code VARCHAR(6) NOT NULL,
    purpose verification_purpose NOT NULL DEFAULT 'REGISTRATION',
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_phone_verifications_lookup ON phone_verifications(phone_number, purpose, is_used);

-- Email OTPs
CREATE TABLE email_otps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    code VARCHAR(6) NOT NULL,
    purpose verification_purpose NOT NULL DEFAULT 'REGISTRATION',
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_email_otps_lookup ON email_otps(email, purpose, is_used);

-- WhatsApp OTPs
CREATE TABLE whatsapp_otps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number VARCHAR(15) NOT NULL,
    code VARCHAR(6) NOT NULL,
    purpose verification_purpose NOT NULL DEFAULT 'REGISTRATION',
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

-- Property Information
CREATE TABLE properties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(200) NOT NULL,
    description TEXT,
    property_type property_type,
    price DECIMAL(12,2),
    subscription_status subscription_status NOT NULL DEFAULT 'inactive',
    subscription_tier VARCHAR(20),
    subscription_start_date TIMESTAMPTZ,
    subscription_end_date TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    owner_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    status property_status NOT NULL DEFAULT 'available',
    county VARCHAR(100),
    location VARCHAR(200),
    plot_number VARCHAR(50),
    constituency VARCHAR(100),
    ward VARCHAR(100),
    purpose VARCHAR(100),
    general_features JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_properties_owner ON properties(owner_id);
CREATE INDEX idx_properties_county ON properties(county);
CREATE INDEX idx_properties_status ON properties(status);
CREATE INDEX idx_properties_subscription ON properties(subscription_status);

-- Property Units
CREATE TABLE property_units (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    property_type unit_type,
    price DECIMAL(10,2),
    status unit_status NOT NULL DEFAULT 'vacant',
    purpose unit_purpose NOT NULL DEFAULT 'rent',
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    floor INTEGER,
    unit_number VARCHAR(50),
    size_sqft DECIMAL(8,2),
    specific_features JSONB,
    total_units INTEGER DEFAULT 1,
    vacant_units INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_property_units_property ON property_units(property_id);

-- Property Images
CREATE TABLE property_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    image_url VARCHAR(500) NOT NULL,
    caption VARCHAR(200),
    is_main BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_property_images_property ON property_images(property_id);

-- Unit Images
CREATE TABLE unit_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    unit_id UUID NOT NULL REFERENCES property_units(id) ON DELETE CASCADE,
    image_url VARCHAR(500) NOT NULL,
    caption VARCHAR(200),
    is_main BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Amenities
CREATE TABLE amenities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    icon VARCHAR(50),
    description TEXT
);

-- Property Amenities
CREATE TABLE property_amenities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    amenity_id UUID NOT NULL REFERENCES amenities(id) ON DELETE CASCADE,
    is_available BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE(property_id, amenity_id)
);

-- Subscription Plans
CREATE TABLE subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    tier plan_tier NOT NULL,
    price DECIMAL(10,2) NOT NULL DEFAULT 0,
    duration plan_duration NOT NULL DEFAULT 'monthly',
    properties_limit INTEGER NOT NULL DEFAULT 1,
    features JSONB,
    max_images_per_property INTEGER NOT NULL DEFAULT 3,
    max_units_per_property INTEGER NOT NULL DEFAULT 1,
    analytics_access BOOLEAN NOT NULL DEFAULT FALSE,
    priority_support BOOLEAN NOT NULL DEFAULT FALSE,
    featured_listing BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Property Subscriptions
CREATE TABLE property_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscription_plans(id) ON DELETE CASCADE,
    status subscription_status NOT NULL DEFAULT 'pending',
    amount_paid DECIMAL(10,2) NOT NULL DEFAULT 0,
    transaction_id VARCHAR(100),
    payment_method VARCHAR(50),
    payment_status payment_status NOT NULL DEFAULT 'pending',
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_property_subscriptions_property ON property_subscriptions(property_id);
CREATE INDEX idx_property_subscriptions_status ON property_subscriptions(status);

-- Insert default subscription plans
INSERT INTO subscription_plans (name, tier, price, duration, properties_limit, max_images_per_property, max_units_per_property, features, is_active) VALUES
('Free Tier', 'free_tier', 0, 'permanent', 1, 3, 1, '["Basic listing", "3 images", "1 unit"]', TRUE),
('Free Trial', 'free_trial', 0, 'trial', 1, 10, 3, '["7-day trial", "All premium features", "Priority support"]', TRUE),
('Basic', 'basic', 1000, 'monthly', 3, 10, 5, '["3 properties", "10 images each", "5 units each", "Basic analytics"]', TRUE),
('Professional', 'professional', 3000, 'monthly', 10, 50, 20, '["10 properties", "50 images each", "20 units each", "Advanced analytics", "Priority support"]', TRUE),
('Premium', 'premium', 5000, 'monthly', 50, 100, 50, '["50 properties", "100 images each", "50 units each", "Full analytics", "Priority support", "Featured listings"]', TRUE);
