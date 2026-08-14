-- Track B2C (Business-to-Customer) M-Pesa payout attempts
CREATE TABLE IF NOT EXISTS b2c_payouts (
                                           id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payout_request_id UUID NOT NULL REFERENCES payout_requests(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    amount DECIMAL(15, 2) NOT NULL,
    phone_number VARCHAR(20) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'queued'
    CHECK (status IN ('queued', 'processing', 'sent', 'delivered', 'failed', 'cancelled')),
    conversation_id VARCHAR(100),
    originator_conversation_id VARCHAR(100),
    result_code VARCHAR(10),
    result_description TEXT,
    retry_count INTEGER DEFAULT 0,
    last_attempt_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

CREATE INDEX IF NOT EXISTS idx_b2c_payouts_agent ON b2c_payouts(agent_id);
CREATE INDEX IF NOT EXISTS idx_b2c_payouts_status ON b2c_payouts(status);
CREATE INDEX IF NOT EXISTS idx_b2c_payouts_payout_request ON b2c_payouts(payout_request_id);

COMMENT ON TABLE b2c_payouts IS 'Tracks automated M-Pesa B2C payout attempts for approved payout requests';