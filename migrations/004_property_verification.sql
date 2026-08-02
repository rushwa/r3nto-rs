-- Add verification fields to properties table
ALTER TABLE properties 
ADD COLUMN IF NOT EXISTS video_url TEXT,
ADD COLUMN IF NOT EXISTS latitude DECIMAL(10, 8),
ADD COLUMN IF NOT EXISTS longitude DECIMAL(11, 8),
ADD COLUMN IF NOT EXISTS verified_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS first_viewed_at TIMESTAMPTZ;

CREATE INDEX idx_properties_verified_at ON properties(verified_at);
CREATE INDEX idx_properties_subscription_status ON properties(subscription_status);
