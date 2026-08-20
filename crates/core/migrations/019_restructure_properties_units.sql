-- ═══════════════════════════════════════════
-- RESTRUCTURE: Property = purpose + location
--              Unit    = price + bedrooms + bathrooms
--              Land    = price + plot size (no units)
-- ═══════════════════════════════════════════

-- 1. Add purpose + land fields to properties
ALTER TABLE properties
    ADD COLUMN IF NOT EXISTS purpose VARCHAR(50) DEFAULT 'for_rent',
    ADD COLUMN IF NOT EXISTS is_land BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS plot_size VARCHAR(100),
    ADD COLUMN IF NOT EXISTS plot_dimensions VARCHAR(50),
    ADD COLUMN IF NOT EXISTS land_price DECIMAL(14, 2);

-- 2. Make unit-specific fields nullable on properties (they belong to units now)
ALTER TABLE properties
    ALTER COLUMN price DROP NOT NULL,
ALTER COLUMN bedrooms DROP NOT NULL,
    ALTER COLUMN bathrooms DROP NOT NULL,
    ALTER COLUMN area_sqft DROP NOT NULL;

-- 3. Add purpose to units (a unit can be rent OR sale, even if building is mixed)
ALTER TABLE property_units
    ADD COLUMN IF NOT EXISTS purpose VARCHAR(50) DEFAULT 'for_rent';

-- 4. Add index for fast lookups
CREATE INDEX IF NOT EXISTS idx_properties_purpose ON properties(purpose);
CREATE INDEX IF NOT EXISTS idx_units_purpose ON property_units(purpose);

-- 5. Seed Kenyan unit features (if not already seeded)
INSERT INTO unit_features (name, category, icon, description) VALUES
                                                                  ('Water', 'utility', '💧', 'Reliable water supply'),
                                                                  ('Electricity', 'utility', '⚡', 'Grid electricity (KPLC)'),
                                                                  ('Solar Power', 'utility', '☀️', 'Solar panel installation'),
                                                                  ('Backup Generator', 'utility', '🔋', 'Backup power generator'),
                                                                  ('Internet/WiFi', 'utility', '📶', 'Internet connectivity'),
                                                                  ('Borehole', 'utility', '🕳️', 'On-site borehole'),
                                                                  ('24/7 Security', 'security', '🛡️', 'Round-the-clock security'),
                                                                  ('CCTV', 'security', '📹', 'CCTV surveillance'),
                                                                  ('Electric Fence', 'security', '⚡', 'Perimeter electric fence'),
                                                                  ('Secure Parking', 'security', '🅿️', 'Gated/covered parking'),
                                                                  ('Biometric Access', 'security', '🔐', 'Fingerprint/keycard access'),
                                                                  ('Swimming Pool', 'amenity', '🏊', 'Swimming pool access'),
                                                                  ('Gym', 'amenity', '💪', 'On-site gym'),
                                                                  ('Garden', 'amenity', '🌳', 'Garden/green space'),
                                                                  ('Balcony', 'amenity', '🌅', 'Private balcony'),
                                                                  ('Elevator', 'amenity', '🛗', 'Building elevator'),
                                                                  ('Parking Space', 'parking', '🚗', 'Dedicated parking slot'),
                                                                  ('Wheelchair Access', 'accessibility', '♿', 'Wheelchair accessible')
    ON CONFLICT (name) DO NOTHING;