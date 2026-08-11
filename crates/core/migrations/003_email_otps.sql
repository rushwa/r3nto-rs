-- Email OTPs Table (for role conversion)
-- Note: email_otps table may already exist from migration 1 with different schema
-- We'll add the missing columns if they don't exist

-- Add user_id column if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name='email_otps' AND column_name='user_id') THEN
ALTER TABLE email_otps ADD COLUMN user_id UUID REFERENCES account_users(id) ON DELETE CASCADE;
END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name='email_otps' AND column_name='otp') THEN
ALTER TABLE email_otps ADD COLUMN otp VARCHAR(6);
END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name='email_otps' AND column_name='expires_at') THEN
ALTER TABLE email_otps ADD COLUMN expires_at TIMESTAMPTZ;
END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name='email_otps' AND column_name='used_at') THEN
ALTER TABLE email_otps ADD COLUMN used_at TIMESTAMPTZ;
END IF;
END $$;

-- Create indexes (only if they don't exist)
CREATE INDEX IF NOT EXISTS idx_email_otps_user_id ON email_otps(user_id);
CREATE INDEX IF NOT EXISTS idx_email_otps_expires_at ON email_otps(expires_at);