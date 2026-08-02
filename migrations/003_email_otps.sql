-- Email OTPs Table
CREATE TABLE IF NOT EXISTS email_otps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES account_users(id) ON DELETE CASCADE,
    otp VARCHAR(6) NOT NULL,
    purpose VARCHAR(50) NOT NULL DEFAULT 'role_conversion',
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_otps_user_id ON email_otps(user_id);
CREATE INDEX idx_email_otps_expires_at ON email_otps(expires_at);
