-- Migration 009: Create payout_requests table
-- This is ADDITIVE — it only creates the table if it doesn't exist

CREATE TABLE IF NOT EXISTS payout_requests (
                                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    amount DECIMAL(15, 2) NOT NULL CHECK (amount > 0),
    mpesa_phone VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'approved', 'rejected')),
    admin_notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE
                                                            );

-- Indexes (IF NOT EXISTS makes these safe to re-run)
CREATE INDEX IF NOT EXISTS idx_payout_requests_agent_id ON payout_requests(agent_id);
CREATE INDEX IF NOT EXISTS idx_payout_requests_status ON payout_requests(status);
CREATE INDEX IF NOT EXISTS idx_payout_requests_created_at ON payout_requests(created_at DESC);

COMMENT ON TABLE payout_requests IS 'Agent payout requests with admin approval workflow';