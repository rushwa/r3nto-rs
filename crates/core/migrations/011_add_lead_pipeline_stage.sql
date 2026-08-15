-- Add pipeline stage to agent leads for CRM workflow
ALTER TABLE agent_leads ADD COLUMN IF NOT EXISTS pipeline_stage VARCHAR(50) DEFAULT 'new';

-- Drop the constraint if it already exists (idempotent)
ALTER TABLE agent_leads DROP CONSTRAINT IF EXISTS chk_pipeline_stage;

-- Re-add the check constraint (safe to run multiple times)
ALTER TABLE agent_leads ADD CONSTRAINT chk_pipeline_stage
    CHECK (pipeline_stage IN ('new', 'contacted', 'viewing_scheduled', 'negotiation', 'closed', 'lost'));

-- Index for faster filtering by stage (idempotent)
CREATE INDEX IF NOT EXISTS idx_agent_leads_stage ON agent_leads(pipeline_stage);

COMMENT ON COLUMN agent_leads.pipeline_stage IS 'Current stage in the sales pipeline: new, contacted, viewing_scheduled, negotiation, closed, lost';