-- Tenant info table schema
-- Contains the business data for each tenant
-- This table is created in each tenant's individual schema

CREATE TABLE IF NOT EXISTS tenant_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create trigger for automatic updated_at management
DROP TRIGGER IF EXISTS update_tenant_info_updated_at ON tenant_info;

CREATE TRIGGER update_tenant_info_updated_at
    BEFORE UPDATE ON tenant_info
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column(); 