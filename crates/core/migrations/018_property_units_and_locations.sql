-- ═══════════════════════════════════════════
-- HIERARCHICAL LOCATION SYSTEM (Kenya-focused)
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS locations (
                                         id SERIAL PRIMARY KEY,
                                         parent_id INTEGER REFERENCES locations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    level VARCHAR(50) NOT NULL CHECK (level IN ('country', 'county', 'constituency', 'ward', 'location', 'village')),
    code VARCHAR(20),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(parent_id, name, level)
    );

CREATE INDEX IF NOT EXISTS idx_locations_level ON locations(level);
CREATE INDEX IF NOT EXISTS idx_locations_parent ON locations(parent_id);

-- ═══════════════════════════════════════════
-- PROPERTY UNITS
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS property_units (
                                              id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    unit_number VARCHAR(50) NOT NULL,
    unit_type VARCHAR(50) NOT NULL DEFAULT 'apartment',
    purpose VARCHAR(50) DEFAULT 'for_rent',
    bedrooms INTEGER DEFAULT 0,
    bathrooms INTEGER DEFAULT 0,
    area_sqft INTEGER,
    price DECIMAL(12, 2),
    status VARCHAR(50) DEFAULT 'available',
    floor_number INTEGER DEFAULT 0,
    description TEXT,
    features JSONB DEFAULT '{}'::jsonb,
    images TEXT[] DEFAULT '{}'::text[],
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(property_id, unit_number)
    );

CREATE INDEX IF NOT EXISTS idx_units_property ON property_units(property_id);
CREATE INDEX IF NOT EXISTS idx_units_status ON property_units(status);
CREATE INDEX IF NOT EXISTS idx_units_purpose ON property_units(purpose);

-- ═══════════════════════════════════════════
-- UNIT FEATURES
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS unit_features (
                                             id SERIAL PRIMARY KEY,
                                             name VARCHAR(100) NOT NULL UNIQUE,
    category VARCHAR(50) NOT NULL,
    icon VARCHAR(10),
    description TEXT,
    is_active BOOLEAN DEFAULT TRUE
    );

-- ═══════════════════════════════════════════
-- ADD GEOLOCATION TO PROPERTIES
-- ═══════════════════════════════════════════

ALTER TABLE properties
    ADD COLUMN IF NOT EXISTS latitude DECIMAL(10, 8),
    ADD COLUMN IF NOT EXISTS longitude DECIMAL(11, 8),
    ADD COLUMN IF NOT EXISTS village VARCHAR(255),
    ADD COLUMN IF NOT EXISTS map_address TEXT,
    ADD COLUMN IF NOT EXISTS geocoded_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_properties_geo ON properties(latitude, longitude);

-- ═══════════════════════════════════════════
-- SEED: UNIT FEATURES
-- ═══════════════════════════════════════════

INSERT INTO unit_features (name, category, icon, description) VALUES
                                                                  ('Water', 'utility', '💧', 'Reliable water supply'),
                                                                  ('Electricity', 'utility', '⚡', 'Grid electricity connection'),
                                                                  ('Solar Power', 'utility', '☀️', 'Solar panel installation'),
                                                                  ('Backup Generator', 'utility', '🔋', 'Backup power generator'),
                                                                  ('Internet/WiFi', 'utility', '📶', 'Internet connectivity available'),
                                                                  ('Borehole', 'utility', '🕳️', 'On-site borehole'),
                                                                  ('24/7 Security', 'security', '🛡️', 'Round-the-clock security guards'),
                                                                  ('CCTV', 'security', '📹', 'CCTV surveillance system'),
                                                                  ('Electric Fence', 'security', '⚡', 'Perimeter electric fence'),
                                                                  ('Secure Parking', 'security', '🅿️', 'Gated/covered parking'),
                                                                  ('Intercom', 'security', '📞', 'Intercom system'),
                                                                  ('Biometric Access', 'security', '🔐', 'Fingerprint/keycard access'),
                                                                  ('Swimming Pool', 'amenity', '🏊', 'Swimming pool access'),
                                                                  ('Gym', 'amenity', '💪', 'On-site gym facility'),
                                                                  ('Garden', 'amenity', '🌳', 'Garden/green space'),
                                                                  ('Playground', 'amenity', '🎪', 'Children playground'),
                                                                  ('Rooftop', 'amenity', '🏙️', 'Rooftop access/terrace'),
                                                                  ('Laundry Area', 'amenity', '🧺', 'Dedicated laundry space'),
                                                                  ('Balcony', 'amenity', '🌅', 'Private balcony'),
                                                                  ('Elevator', 'amenity', '🛗', 'Building elevator'),
                                                                  ('Parking Space', 'parking', '🚗', 'Dedicated parking spot'),
                                                                  ('Garage', 'parking', '🏠', 'Private garage'),
                                                                  ('Wheelchair Access', 'accessibility', '♿', 'Wheelchair accessible'),
                                                                  ('Ground Floor', 'accessibility', '🚪', 'Ground floor unit')
    ON CONFLICT (name) DO NOTHING;