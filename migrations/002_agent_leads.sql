-- Agent Leads Table
CREATE TYPE lead_status AS ENUM ('pending', 'converted', 'rejected');

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

CREATE INDEX idx_agent_leads_status ON agent_leads(status);
CREATE INDEX idx_agent_leads_claimed_by ON agent_leads(claimed_by);
