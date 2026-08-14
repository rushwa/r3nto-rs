-- Add pipeline stage to agent leads for CRM workflow
ALTER TABLE agent_leads ADD COLUMN IF NOT EXISTS pipeline_stage VARCHAR(50) DEFAULT 'new';

-- Add a check constraint to ensure valid stages
ALTER TABLE agent_leads ADD CONSTRAINT chk_pipeline_stage
    CHECK (pipeline_stage IN ('new', 'contacted', 'viewing_scheduled', 'negotiation', 'closed', 'lost'));

-- Index for faster filtering by stage
CREATE INDEX IF NOT EXISTS idx_agent_leads_stage ON agent_leads(pipeline_stage);

COMMENT ON COLUMN agent_leads.pipeline_stage IS 'Current stage in the sales pipeline: new, contacted, viewing_scheduled, negotiation, closed, lost';