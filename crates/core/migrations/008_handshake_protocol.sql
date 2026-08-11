-- migrations/008_handshake_protocol.sql

-- 1. Add ROLE_CONVERSION to the verification_purpose enum safely
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_enum
        WHERE enumtypid = 'verification_purpose'::regtype
        AND enumlabel = 'ROLE_CONVERSION'
    ) THEN
ALTER TYPE verification_purpose ADD VALUE 'ROLE_CONVERSION';
END IF;
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 2. Create table to track Agent-Owner conversion relationships
CREATE TABLE IF NOT EXISTS agent_conversions (
                                                 id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    property_owner_id UUID NOT NULL UNIQUE REFERENCES account_users(id) ON DELETE CASCADE,
    converted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    renewal_commission_split DECIMAL(5,2) NOT NULL DEFAULT 10.00
    );

-- 3. Create indexes IF NOT EXISTS to prevent "already exists" errors
CREATE INDEX IF NOT EXISTS idx_agent_conversions_agent ON agent_conversions(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_conversions_owner ON agent_conversions(property_owner_id);