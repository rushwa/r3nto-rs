-- Add unique index on email for email_otps to support ON CONFLICT
CREATE UNIQUE INDEX IF NOT EXISTS idx_email_otps_email ON email_otps(email);