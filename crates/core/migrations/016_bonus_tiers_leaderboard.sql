-- ═══════════════════════════════════════════
-- REFERRAL BONUS TIERS
-- ═══════════════════════════════════════════

-- Define bonus tier thresholds
CREATE TABLE IF NOT EXISTS referral_bonus_tiers (
                                                    id SERIAL PRIMARY KEY,
                                                    tier_name VARCHAR(50) NOT NULL,
    min_referrals INTEGER NOT NULL,
    bonus_amount DECIMAL(15, 2) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(min_referrals)
    );

-- Seed default tiers
INSERT INTO referral_bonus_tiers (tier_name, min_referrals, bonus_amount) VALUES
                                                                              ('Bronze', 5, 500.00),
                                                                              ('Silver', 10, 1500.00),
                                                                              ('Gold', 25, 5000.00),
                                                                              ('Platinum', 50, 15000.00),
                                                                              ('Diamond', 100, 50000.00)
    ON CONFLICT (min_referrals) DO NOTHING;

-- Track which bonuses an agent has already claimed (prevent double-claiming)
CREATE TABLE IF NOT EXISTS agent_bonus_claims (
                                                  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    tier_id INTEGER NOT NULL REFERENCES referral_bonus_tiers(id) ON DELETE CASCADE,
    bonus_amount DECIMAL(15, 2) NOT NULL,
    referral_count_at_claim INTEGER NOT NULL,
    claimed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(agent_id, tier_id)
    );

CREATE INDEX IF NOT EXISTS idx_bonus_claims_agent ON agent_bonus_claims(agent_id);

COMMENT ON TABLE referral_bonus_tiers IS 'Defines milestone thresholds for referral bonuses';
COMMENT ON TABLE agent_bonus_claims IS 'Tracks which tier bonuses each agent has already received';

-- ═══════════════════════════════════════════
-- LEADERBOARD SNAPSHOT (refreshed periodically)
-- ═══════════════════════════════════════════

CREATE TABLE IF NOT EXISTS agent_leaderboard_cache (
                                                       agent_id UUID PRIMARY KEY REFERENCES account_users(id) ON DELETE CASCADE,
    agent_name VARCHAR(255) NOT NULL,
    total_conversions INTEGER DEFAULT 0,
    total_commissions DECIMAL(15, 2) DEFAULT 0,
    total_referrals INTEGER DEFAULT 0,
    properties_managed INTEGER DEFAULT 0,
    leads_closed INTEGER DEFAULT 0,
    score DECIMAL(15, 2) DEFAULT 0,
    rank INTEGER DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

CREATE INDEX IF NOT EXISTS idx_leaderboard_score ON agent_leaderboard_cache(score DESC);

COMMENT ON TABLE agent_leaderboard_cache IS 'Cached leaderboard rankings, refreshed on each query';