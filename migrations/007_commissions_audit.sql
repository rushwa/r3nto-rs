-- Commissions Audit Table Enhancement
-- Add missing columns to existing commissions table if they don't exist

-- Add property_id column if it doesn't exist
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='commissions' AND column_name='property_id') THEN
        ALTER TABLE commissions ADD COLUMN property_id UUID REFERENCES properties(id) ON DELETE CASCADE;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='commissions' AND column_name='transaction_ref') THEN
        ALTER TABLE commissions ADD COLUMN transaction_ref VARCHAR(255) UNIQUE;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='commissions' AND column_name='commission_rate') THEN
        ALTER TABLE commissions ADD COLUMN commission_rate DECIMAL(5, 2) NOT NULL DEFAULT 30.00;
    END IF;
END $$;

-- Create indexes if they don't exist
CREATE INDEX IF NOT EXISTS idx_commissions_property_id ON commissions(property_id);
CREATE INDEX IF NOT EXISTS idx_commissions_transaction_ref ON commissions(transaction_ref);
