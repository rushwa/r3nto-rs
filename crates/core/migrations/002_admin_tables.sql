-- migrations/002_admin_tables.sql
-- Admin panel, system settings, and M-Pesa financial infrastructure

-- 1. System Settings (with registration_fee)
CREATE TABLE IF NOT EXISTS system_settings (
                                               id INT PRIMARY KEY DEFAULT 1,
                                               company_name VARCHAR(255) NOT NULL DEFAULT 'Rento',
    commission_rate DECIMAL(5,2) NOT NULL DEFAULT 2.5,
    registration_fee DECIMAL(10,2) NOT NULL DEFAULT 1000.00,
    maintenance_mode BOOLEAN NOT NULL DEFAULT FALSE,
    allow_registration BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

INSERT INTO system_settings (id, company_name, commission_rate, registration_fee, maintenance_mode, allow_registration)
VALUES (1, 'Rento', 2.5, 1000.00, FALSE, TRUE)
    ON CONFLICT (id) DO NOTHING;

-- 2. Admin Inquiries
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

-- ==========================================
-- 3. M-Pesa Commissions & Wallet Infrastructure
-- (Merged from original 009 to avoid SQLx parsing issues)
-- ==========================================

-- M-Pesa raw transaction records
CREATE TABLE IF NOT EXISTS mpesa_transactions (
                                                  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_request_id VARCHAR(100) UNIQUE NOT NULL,
    checkout_request_id VARCHAR(100) UNIQUE NOT NULL,
    mpesa_receipt_number VARCHAR(50),
    phone_number VARCHAR(15) NOT NULL,
    amount DECIMAL(12,2) NOT NULL,
    transaction_type VARCHAR(50) NOT NULL DEFAULT 'C2B',
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    result_code INTEGER,
    result_desc TEXT,
    callback_raw JSONB,
    callback_received_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
CREATE INDEX IF NOT EXISTS idx_mpesa_tx_checkout ON mpesa_transactions(checkout_request_id);
CREATE INDEX IF NOT EXISTS idx_mpesa_tx_receipt ON mpesa_transactions(mpesa_receipt_number);
CREATE INDEX IF NOT EXISTS idx_mpesa_tx_status ON mpesa_transactions(status);

-- Business-level payment records
CREATE TABLE IF NOT EXISTS payments (
                                        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payer_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    mpesa_transaction_id UUID REFERENCES mpesa_transactions(id),
    payment_type VARCHAR(50) NOT NULL,
    reference_id UUID,
    amount DECIMAL(12,2) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
CREATE INDEX IF NOT EXISTS idx_payments_payer ON payments(payer_id);
CREATE INDEX IF NOT EXISTS idx_payments_type ON payments(payment_type);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);

-- Commission ledger
CREATE TABLE IF NOT EXISTS commission_ledger (
                                                 id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    payment_id UUID NOT NULL REFERENCES payments(id) ON DELETE CASCADE,
    property_owner_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    property_id UUID REFERENCES properties(id) ON DELETE SET NULL,
    commission_type VARCHAR(50) NOT NULL,
    gross_amount DECIMAL(12,2) NOT NULL,
    commission_rate DECIMAL(5,2) NOT NULL,
    commission_amount DECIMAL(12,2) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    credited_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
CREATE INDEX IF NOT EXISTS idx_commission_ledger_agent ON commission_ledger(agent_id);
CREATE INDEX IF NOT EXISTS idx_commission_ledger_payment ON commission_ledger(payment_id);
CREATE INDEX IF NOT EXISTS idx_commission_ledger_status ON commission_ledger(status);

-- Agent wallets
CREATE TABLE IF NOT EXISTS agent_wallets (
                                             id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL UNIQUE REFERENCES account_users(id) ON DELETE CASCADE,
    balance DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    pending_balance DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    total_earned DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    total_withdrawn DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    mpesa_phone VARCHAR(15),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
CREATE INDEX IF NOT EXISTS idx_agent_wallets_agent ON agent_wallets(agent_id);

-- Wallet transaction audit trail
CREATE TABLE IF NOT EXISTS wallet_transactions (
                                                   id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES agent_wallets(id) ON DELETE CASCADE,
    transaction_type VARCHAR(50) NOT NULL,
    amount DECIMAL(12,2) NOT NULL,
    balance_before DECIMAL(12,2) NOT NULL,
    balance_after DECIMAL(12,2) NOT NULL,
    reference VARCHAR(255),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
CREATE INDEX IF NOT EXISTS idx_wallet_tx_wallet ON wallet_transactions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_wallet_tx_type ON wallet_transactions(transaction_type);

-- Payout requests
CREATE TABLE IF NOT EXISTS payout_requests (
                                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    wallet_id UUID NOT NULL REFERENCES agent_wallets(id) ON DELETE CASCADE,
    amount DECIMAL(12,2) NOT NULL,
    mpesa_phone VARCHAR(15) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    mpesa_transaction_id UUID REFERENCES mpesa_transactions(id),
    admin_notes TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
    );
CREATE INDEX IF NOT EXISTS idx_payout_requests_agent ON payout_requests(agent_id);
CREATE INDEX IF NOT EXISTS idx_payout_requests_status ON payout_requests(status);