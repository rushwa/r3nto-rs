-- ═══════════════════════════════════════════
-- VIRTUAL TOUR REQUESTS (20 KES fee, 24-hour SLA)
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS virtual_tour_requests (
                                                     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    client_id UUID REFERENCES account_users(id) ON DELETE SET NULL,
    client_email VARCHAR(255) NOT NULL,
    client_name VARCHAR(255),
    client_phone VARCHAR(50),
    fee_amount DECIMAL(10, 2) NOT NULL DEFAULT 20.00,
    fee_paid BOOLEAN DEFAULT FALSE,
    payment_reference VARCHAR(100),
    status VARCHAR(30) NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'fulfilled', 'expired', 'cancelled', 'property_delisted')),
    assigned_agent_id UUID REFERENCES account_users(id) ON DELETE SET NULL,
    sla_deadline TIMESTAMP WITH TIME ZONE,
    fulfilled_at TIMESTAMP WITH TIME ZONE,
    expired_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

CREATE INDEX IF NOT EXISTS idx_tour_requests_property ON virtual_tour_requests(property_id);
CREATE INDEX IF NOT EXISTS idx_tour_requests_client ON virtual_tour_requests(client_id);
CREATE INDEX IF NOT EXISTS idx_tour_requests_agent ON virtual_tour_requests(assigned_agent_id);
CREATE INDEX IF NOT EXISTS idx_tour_requests_status ON virtual_tour_requests(status);
CREATE INDEX IF NOT EXISTS idx_tour_requests_sla ON virtual_tour_requests(sla_deadline) WHERE status = 'pending';

COMMENT ON TABLE virtual_tour_requests IS 'Client requests for virtual property tours (20 KES fee, 24-hour SLA)';

-- ═══════════════════════════════════════════
-- VIRTUAL TOUR VIDEOS (with watermarking metadata)
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS virtual_tour_videos (
                                                   id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tour_request_id UUID NOT NULL REFERENCES virtual_tour_requests(id) ON DELETE CASCADE,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    video_url TEXT NOT NULL,
    thumbnail_url TEXT,
    duration_seconds INTEGER,
    file_size_bytes BIGINT,
    -- Watermark metadata (embedded in video)
    watermark_agent_id VARCHAR(50) NOT NULL,
    watermark_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                                                                           watermark_logo_applied BOOLEAN DEFAULT TRUE,
                                                                           -- Recording metadata (proves native capture)
                                                                           recorded_via VARCHAR(50) DEFAULT 'native_in_app',
    device_fingerprint VARCHAR(255),
    recording_started_at TIMESTAMP WITH TIME ZONE,
    recording_completed_at TIMESTAMP WITH TIME ZONE,
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

CREATE INDEX IF NOT EXISTS idx_tour_videos_request ON virtual_tour_videos(tour_request_id);
CREATE INDEX IF NOT EXISTS idx_tour_videos_property ON virtual_tour_videos(property_id);
CREATE INDEX IF NOT EXISTS idx_tour_videos_agent ON virtual_tour_videos(agent_id);

COMMENT ON TABLE virtual_tour_videos IS 'Native in-app recorded videos with automatic watermarking';

-- ═══════════════════════════════════════════
-- TOUR VIEWING SESSIONS (2-hour rule + device lock)
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS tour_viewing_sessions (
                                                     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tour_request_id UUID NOT NULL REFERENCES virtual_tour_requests(id) ON DELETE CASCADE,
    video_id UUID NOT NULL REFERENCES virtual_tour_videos(id) ON DELETE CASCADE,
    client_id UUID REFERENCES account_users(id) ON DELETE SET NULL,
    viewing_token VARCHAR(255) NOT NULL UNIQUE,
    -- 2-hour rule
    viewing_started_at TIMESTAMP WITH TIME ZONE,
    viewing_expires_at TIMESTAMP WITH TIME ZONE,
                                                                           -- Device locking
                                                                           device_fingerprint VARCHAR(255),
    device_locked BOOLEAN DEFAULT FALSE,
    locked_at TIMESTAMP WITH TIME ZONE,
    -- Access tracking
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMP WITH TIME ZONE,
                                                                           ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

CREATE INDEX IF NOT EXISTS idx_viewing_sessions_token ON tour_viewing_sessions(viewing_token);
CREATE INDEX IF NOT EXISTS idx_viewing_sessions_request ON tour_viewing_sessions(tour_request_id);
CREATE INDEX IF NOT EXISTS idx_viewing_sessions_expires ON tour_viewing_sessions(viewing_expires_at);

COMMENT ON TABLE tour_viewing_sessions IS 'Secure viewing sessions with 2-hour expiry and device locking';

-- ═══════════════════════════════════════════
-- PROPERTY DE-LISTING FLAG
-- ═══════════════════════════════════════════

ALTER TABLE properties ADD COLUMN IF NOT EXISTS is_delisted BOOLEAN DEFAULT FALSE;
ALTER TABLE properties ADD COLUMN IF NOT EXISTS delisted_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE properties ADD COLUMN IF NOT EXISTS delisted_reason TEXT;

CREATE INDEX IF NOT EXISTS idx_properties_delisted ON properties(is_delisted) WHERE is_delisted = TRUE;

COMMENT ON COLUMN properties.is_delisted IS 'True if property is no longer available (prevents new tour requests)';

-- ═══════════════════════════════════════════
-- AGENT SLA PERFORMANCE TRACKING
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS agent_sla_metrics (
                                                 agent_id UUID PRIMARY KEY REFERENCES account_users(id) ON DELETE CASCADE,
    total_tours_assigned INTEGER DEFAULT 0,
    tours_fulfilled_on_time INTEGER DEFAULT 0,
    tours_fulfilled_late INTEGER DEFAULT 0,
    tours_expired INTEGER DEFAULT 0,
    average_fulfillment_minutes INTEGER DEFAULT 0,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

COMMENT ON TABLE agent_sla_metrics IS 'Tracks agent performance on 24-hour tour fulfillment SLA';