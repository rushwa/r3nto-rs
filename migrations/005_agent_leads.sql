-- Agent Leads Table
-- Tracks potential property owners that agents can claim and convert

-- Create lead_status enum if it doesn't exist
DO $$ BEGIN
    CREATE TYPE lead_status AS ENUM ('pending', 'converted', 'rejected');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create agent_leads table if it doesn't exist
CREATE TABLE IF NOT EXISTS agent_leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    full_name VARCHAR(255) NOT NULL,
    phone VARCHAR(50),
    status lead_status NOT NULL DEFAULT 'pending',
    claimed_by UUID REFERENCES account_users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes if they don't exist
CREATE INDEX IF NOT EXISTS idx_agent_leads_status ON agent_leads(status);
CREATE INDEX IF NOT EXISTS idx_agent_leads_claimed_by ON agent_leads(claimed_by);
CREATE INDEX IF NOT EXISTS idx_agent_leads_email ON agent_leads(email);
