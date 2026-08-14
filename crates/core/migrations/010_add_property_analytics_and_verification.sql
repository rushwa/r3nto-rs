-- Add analytics columns to properties
ALTER TABLE properties ADD COLUMN IF NOT EXISTS views_count INTEGER DEFAULT 0;
ALTER TABLE properties ADD COLUMN IF NOT EXISTS inquiries_count INTEGER DEFAULT 0;
ALTER TABLE properties ADD COLUMN IF NOT EXISTS favorites_count INTEGER DEFAULT 0;

-- Add verification status to account_users (for "Verified Owner" badge)
ALTER TABLE account_users ADD COLUMN IF NOT EXISTS is_verified BOOLEAN DEFAULT FALSE;
ALTER TABLE account_users ADD COLUMN IF NOT EXISTS verification_document_url TEXT;

-- Add comments for clarity
COMMENT ON COLUMN properties.views_count IS 'Number of times the property listing was viewed';
COMMENT ON COLUMN properties.inquiries_count IS 'Number of inquiries received for this property';
COMMENT ON COLUMN account_users.is_verified IS 'True if the property owner has submitted and passed ID/Title deed verification';