-- Master tenants table schema
-- Only contains identification and timestamps, no business data
-- This table is located in the 'master' schema

CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create trigger for automatic updated_at management
DROP TRIGGER IF EXISTS update_tenants_updated_at ON tenants;

CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column(); 