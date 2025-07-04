-- Tenants schema and table
-- Creates the dedicated "tenants" schema with the main tenants table
-- Uses Row-Level Security instead of schema-per-tenant approach

-- Create the tenants schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS tenants;

-- Create the main tenants table in the tenants schema
CREATE TABLE IF NOT EXISTS tenants.tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    settings JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create trigger for automatic updated_at management
DROP TRIGGER IF EXISTS update_tenants_updated_at ON tenants.tenants;

CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON tenants.tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Enable Row Level Security
ALTER TABLE tenants.tenants ENABLE ROW LEVEL SECURITY;

-- Create index for better performance with tenant_id queries
CREATE INDEX IF NOT EXISTS idx_tenants_name ON tenants.tenants(name);
CREATE INDEX IF NOT EXISTS idx_tenants_active ON tenants.tenants(is_active);
CREATE INDEX IF NOT EXISTS idx_tenants_created_at ON tenants.tenants(created_at); 