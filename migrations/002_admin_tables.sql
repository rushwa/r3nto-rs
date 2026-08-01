-- Admin panel specific tables (not in core schema)

CREATE TABLE IF NOT EXISTS system_settings (
                                               id INT PRIMARY KEY DEFAULT 1,
                                               company_name VARCHAR(255) NOT NULL DEFAULT 'Rento',
    commission_rate DECIMAL(5,2) NOT NULL DEFAULT 2.5,
    maintenance_mode BOOLEAN NOT NULL DEFAULT FALSE,
    allow_registration BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

INSERT INTO system_settings (id, company_name, commission_rate, maintenance_mode, allow_registration)
VALUES (1, 'Rento', 2.5, FALSE, TRUE)
    ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS admin_inquiries (
                                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(50),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    property_title VARCHAR(500) NOT NULL DEFAULT '',
    message TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'new',
    assigned_to VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

CREATE INDEX IF NOT EXISTS idx_admin_inquiries_status ON admin_inquiries(status);
CREATE INDEX IF NOT EXISTS idx_admin_inquiries_property ON admin_inquiries(property_id);