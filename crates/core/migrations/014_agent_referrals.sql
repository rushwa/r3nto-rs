-- Track referral sign-ups for agents
CREATE TABLE IF NOT EXISTS agent_referrals (
                                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    referred_user_id UUID REFERENCES account_users(id) ON DELETE SET NULL,
    referred_email VARCHAR(255) NOT NULL,
    referred_name VARCHAR(255),
    signup_completed BOOLEAN DEFAULT FALSE,
    conversion_completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    converted_at TIMESTAMP WITH TIME ZONE,
                                                            UNIQUE(agent_id, referred_email)
    );

CREATE INDEX IF NOT EXISTS idx_agent_referrals_agent ON agent_referrals(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_referrals_email ON agent_referrals(referred_email);

COMMENT ON TABLE agent_referrals IS 'Tracks referrals brought in by agents via unique referral links';