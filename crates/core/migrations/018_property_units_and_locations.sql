-- ═══════════════════════════════════════════
-- HIERARCHICAL LOCATION SYSTEM (Kenya-focused)
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS locations (
                                         id SERIAL PRIMARY KEY,
                                         parent_id INTEGER REFERENCES locations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    level VARCHAR(50) NOT NULL CHECK (level IN ('country', 'county', 'constituency', 'ward', 'location', 'village')),
    code VARCHAR(20),  -- e.g., county code
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(parent_id, name, level)
    );

CREATE INDEX idx_locations_level ON locations(level);
CREATE INDEX idx_locations_parent ON locations(parent_id);

-- ═══════════════════════════════════════════
-- PROPERTY UNITS (apartments, rooms, etc.)
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS property_units (
                                              id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    unit_number VARCHAR(50) NOT NULL,  -- e.g., "A1", "Ground Floor", "Penthouse"
    unit_type VARCHAR(50) NOT NULL DEFAULT 'apartment',  -- apartment, bedsitter, single, commercial, etc.
    bedrooms INTEGER DEFAULT 0,
    bathrooms INTEGER DEFAULT 0,
    area_sqft INTEGER,
    price DECIMAL(12, 2),  -- Rent or sale price for this unit
    status VARCHAR(50) DEFAULT 'available' CHECK (status IN ('available', 'occupied', 'reserved', 'maintenance')),
    floor_number INTEGER DEFAULT 0,
    description TEXT,
    features JSONB DEFAULT '{}'::jsonb,  -- {"water": true, "security": true, "parking": 2, "generator": false}
    images TEXT[] DEFAULT '{}'::text[],
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(property_id, unit_number)
    );

CREATE INDEX idx_units_property ON property_units(property_id);
CREATE INDEX idx_units_status ON property_units(status);

-- ═══════════════════════════════════════════
-- UNIT FEATURES (lookup table for available features)
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS unit_features (
                                             id SERIAL PRIMARY KEY,
                                             name VARCHAR(100) NOT NULL UNIQUE,
    category VARCHAR(50) NOT NULL,  -- 'amenity', 'security', 'utility', 'accessibility'
    icon VARCHAR(10),  -- emoji icon
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
    ADD COLUMN IF NOT EXISTS map_address TEXT,  -- Human-readable address for maps
    ADD COLUMN IF NOT EXISTS geocoded_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_properties_geo ON properties(latitude, longitude);

-- ═══════════════════════════════════════════
-- SEED: UNIT FEATURES
-- ═══════════════════════════════════════════

INSERT INTO unit_features (name, category, icon, description) VALUES
                                                                  -- Utilities
                                                                  ('Water', 'utility', '💧', 'Reliable water supply'),
                                                                  ('Electricity', 'utility', '⚡', 'Grid electricity connection'),
                                                                  ('Solar Power', 'utility', '☀️', 'Solar panel installation'),
                                                                  ('Backup Generator', 'utility', '🔋', 'Backup power generator'),
                                                                  ('Internet/WiFi', 'utility', '📶', 'Internet connectivity available'),
                                                                  ('Borehole', 'utility', '🕳️', 'On-site borehole'),

                                                                  -- Security
                                                                  ('24/7 Security', 'security', '🛡️', 'Round-the-clock security guards'),
                                                                  ('CCTV', 'security', '📹', 'CCTV surveillance system'),
                                                                  ('Electric Fence', 'security', '⚡', 'Perimeter electric fence'),
                                                                  ('Secure Parking', 'security', '🅿️', 'Gated/covered parking'),
                                                                  ('Intercom', 'security', '📞', 'Intercom system'),
                                                                  ('Biometric Access', 'security', '🔐', 'Fingerprint/keycard access'),

                                                                  -- Amenities
                                                                  ('Swimming Pool', 'amenity', '🏊', 'Swimming pool access'),
                                                                  ('Gym', 'amenity', '💪', 'On-site gym facility'),
                                                                  ('Garden', 'amenity', '🌳', 'Garden/green space'),
                                                                  ('Playground', 'amenity', '🎪', 'Children playground'),
                                                                  ('Rooftop', 'amenity', '🏙️', 'Rooftop access/terrace'),
                                                                  ('Laundry Area', 'amenity', '🧺', 'Dedicated laundry space'),
                                                                  ('Balcony', 'amenity', '🌅', 'Private balcony'),
                                                                  ('Elevator', 'amenity', '🛗', 'Building elevator'),

                                                                  -- Parking
                                                                  ('Parking Space', 'parking', '🚗', 'Dedicated parking spot'),
                                                                  ('Garage', 'parking', '🏠', 'Private garage'),

                                                                  -- Accessibility
                                                                  ('Wheelchair Access', 'accessibility', '♿', 'Wheelchair accessible'),
                                                                  ('Ground Floor', 'accessibility', '🚪', 'Ground floor unit')
    ON CONFLICT (name) DO NOTHING;

-- ═══════════════════════════════════════════
-- SEED: KENYA LOCATIONS (sample — expand as needed)
-- ═══════════════════════════════════════════

-- Country
INSERT INTO locations (name, level, code) VALUES ('Kenya', 'country', 'KE') ON CONFLICT DO NOTHING;

-- Get Kenya ID for parent references
DO $$
DECLARE
kenya_id INTEGER;
    nairobi_id INTEGER;
    kiambu_id INTEGER;
    mombasa_id INTEGER;
    nakuru_id INTEGER;
BEGIN
SELECT id INTO kenya_id FROM locations WHERE name = 'Kenya' AND level = 'country';

-- Counties
INSERT INTO locations (parent_id, name, level, code) VALUES
                                                         (kenya_id, 'Nairobi', 'county', '047'),
                                                         (kenya_id, 'Kiambu', 'county', '022'),
                                                         (kenya_id, 'Mombasa', 'county', '001'),
                                                         (kenya_id, 'Nakuru', 'county', '031'),
                                                         (kenya_id, 'Kajiado', 'county', '018'),
                                                         (kenya_id, 'Machakos', 'county', '016'),
                                                         (kenya_id, 'Uasin Gishu', 'county', '041'),
                                                         (kenya_id, 'Kisumu', 'county', '042')
    ON CONFLICT DO NOTHING;

-- Nairobi Constituencies
SELECT id INTO nairobi_id FROM locations WHERE name = 'Nairobi' AND level = 'county';
INSERT INTO locations (parent_id, name, level) VALUES
                                                   (nairobi_id, 'Westlands', 'constituency'),
                                                   (nairobi_id, 'Kilimani', 'constituency'),
                                                   (nairobi_id, 'Langata', 'constituency'),
                                                   (nairobi_id, 'Kasarani', 'constituency'),
                                                   (nairobi_id, 'Embakasi', 'constituency'),
                                                   (nairobi_id, 'Kibra', 'constituency'),
                                                   (nairobi_id, 'Starehe', 'constituency'),
                                                   (nairobi_id, 'Kamukunji', 'constituency'),
                                                   (nairobi_id, 'Makadara', 'constituency'),
                                                   (nairobi_id, 'Dagoretti', 'constituency')
    ON CONFLICT DO NOTHING;

-- Kiambu Constituencies
SELECT id INTO kiambu_id FROM locations WHERE name = 'Kiambu' AND level = 'county';
INSERT INTO locations (parent_id, name, level) VALUES
                                                   (kiambu_id, 'Kiambu Town', 'constituency'),
                                                   (kiambu_id, 'Thika Town', 'constituency'),
                                                   (kiambu_id, 'Ruiru', 'constituency'),
                                                   (kiambu_id, 'Kasarani', 'constituency'),
                                                   (kiambu_id, 'Juja', 'constituency'),
                                                   (kiambu_id, 'Gatundu South', 'constituency'),
                                                   (kiambu_id, 'Githunguri', 'constituency'),
                                                   (kiambu_id, 'Kiambaa', 'constituency'),
                                                   (kiambu_id, 'Kabete', 'constituency'),
                                                   (kiambu_id, 'Kikuyu', 'constituency'),
                                                   (kiambu_id, 'Limuru', 'constituency'),
                                                   (kiambu_id, 'Lari', 'constituency')
    ON CONFLICT DO NOTHING;

-- Sample Wards for Westlands
DECLARE
westlands_id INTEGER;
BEGIN
SELECT id INTO westlands_id FROM locations WHERE name = 'Westlands' AND level = 'constituency' AND parent_id = nairobi_id;
INSERT INTO locations (parent_id, name, level) VALUES
                                                   (westlands_id, 'Parklands/Highridge', 'ward'),
                                                   (westlands_id, 'Kangemi', 'ward'),
                                                   (westlands_id, 'Kilimani', 'ward'),
                                                   (westlands_id, 'Kawangware', 'ward'),
                                                   (westlands_id, 'Gatina', 'ward'),
                                                   (westlands_id, 'Kitisuru', 'ward')
    ON CONFLICT DO NOTHING;

-- Sample Locations for Parklands
DECLARE
parklands_id INTEGER;
BEGIN
SELECT id INTO parklands_id FROM locations WHERE name = 'Parklands/Highridge' AND level = 'ward' AND parent_id = westlands_id;
INSERT INTO locations (parent_id, name, level) VALUES
                                                   (parklands_id, 'Parklands', 'location'),
                                                   (parklands_id, 'Westlands CBD', 'location'),
                                                   (parklands_id, 'Spring Valley', 'location'),
                                                   (parklands_id, 'Lower Kabete', 'location'),
                                                   (parklands_id, 'Ruaka', 'location')
    ON CONFLICT DO NOTHING;
END;
END;

    -- Sample Wards for Kiambu Town
    DECLARE
kiambu_town_id INTEGER;
BEGIN
SELECT id INTO kiambu_town_id FROM locations WHERE name = 'Kiambu Town' AND level = 'constituency' AND parent_id = kiambu_id;
INSERT INTO locations (parent_id, name, level) VALUES
                                                   (kiambu_town_id, 'Kiambu Ward', 'ward'),
                                                   (kiambu_town_id, 'Tinganga', 'ward'),
                                                   (kiambu_town_id, 'Ndumberi', 'ward'),
                                                   (kiambu_town_id, 'Riabai', 'ward'),
                                                   (kiambu_town_id, 'Township', 'ward')
    ON CONFLICT DO NOTHING;
END;
END $$;