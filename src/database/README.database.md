# Database Module

Multi-tenant database system using SQLx and PostgreSQL with single schema architecture and Row-Level Security (RLS) for tenant isolation.

## Structure

```
src/database/
├── mod.rs              # Module exports
├── schema_manager.rs   # Database service and connection management
└── sql/schemas/        # SQL schema definitions
```

## Architecture

**Single Schema with Row-Level Security:**
- All tenants share the `tenants` schema for optimal performance
- PostgreSQL RLS policies provide data isolation
- Single connection pool for efficient resource usage
- Unlimited tenant scalability

```
PostgreSQL Database (UTC timezone)
└── Schema: tenants                 
    ├── Table: tenants              (tenant registry)
    ├── Additional tables           (with RLS policies)
    └── Functions & Triggers        (shared utilities)
```

## Features

- **Automatic Initialization** - Schema and table setup on startup
- **Row-Level Security** - PostgreSQL RLS policies for data isolation
- **UTC Timezone** - All connections configured for UTC
- **Standard Fields** - `id`, `created_at`, `updated_at` with triggers
- **Single Connection Pool** - Shared pool for all tenants
- **Thread Safety** - Singleton pattern with `Arc<T>` sharing
- **SSL Configuration** - Environment-based SSL settings
- **Graceful Shutdown** - Controlled connection closure

## Quick Start

### 1. Environment Setup
```env
DB_HOST=localhost
DB_PORT=5432
DB_NAME=rust_axum_api
DB_USER=postgres
DB_PASSWORD=postgres
```

### 2. Start PostgreSQL
```bash
docker-compose -f docker/docker-compose.dev.yml up -d
```

### 3. Run Application
```bash
cargo run
# ✅ Automatic tenants schema initialization
```

## Usage

### Access Database Service
```rust
async fn handler(State(app_state): State<AppState>) -> Result<(), Error> {
    let db = &app_state.database;
    let pool = &db.pool;
    
    // All queries use RLS for tenant isolation
    let tenants = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE is_active = true")
        .fetch_all(pool)
        .await?;
    
    Ok(())
}
```

### Database Operations
```rust
// Create tenant
async fn create_tenant(db: &DatabaseService, name: &str) -> Result<Tenant> {
    let tenant = sqlx::query_as::<_, Tenant>(
        "INSERT INTO tenants (name, settings, is_active) 
         VALUES ($1, $2, $3) 
         RETURNING id, name, settings, is_active, created_at, updated_at"
    )
    .bind(name)
    .bind(serde_json::json!({}))
    .bind(true)
    .fetch_one(&db.pool)
    .await?;
    
    Ok(tenant)
}

// List active tenants
async fn list_tenants(db: &DatabaseService) -> Result<Vec<Tenant>> {
    let tenants = sqlx::query_as::<_, Tenant>(
        "SELECT id, name, settings, is_active, created_at, updated_at 
         FROM tenants 
         WHERE is_active = true 
         ORDER BY created_at DESC"
    )
    .fetch_all(&db.pool)
    .await?;
    
    Ok(tenants)
}
```

## Table Structure

### Main Tenants Table
```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL UNIQUE,
    settings JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Automatic updated_at trigger
CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

## Row-Level Security (RLS)

### RLS Implementation
For tenant-specific tables requiring data isolation:

```sql
-- Enable RLS
ALTER TABLE tenant_data ENABLE ROW LEVEL SECURITY;

-- Create isolation policy
CREATE POLICY tenant_isolation ON tenant_data
    USING (tenant_id = current_setting('rls.tenant_id')::UUID);

-- Grant permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON tenant_data TO app_role;
```

### Setting Tenant Context
```rust
// Set tenant context for RLS
sqlx::query("SELECT set_config('rls.tenant_id', $1, true)")
    .bind(tenant_id.to_string())
    .execute(&pool)
    .await?;

// Now all queries are filtered by RLS policy
```

## Monitoring APIs

```bash
# Database health
curl http://localhost:3000/db/health

# General monitoring
curl http://localhost:3000/db/monitoring

# Tenant metrics
curl http://localhost:3000/db/tenants/monitoring

# Schema information
curl http://localhost:3000/db/schema/monitoring
```

## Benefits of Single Schema + RLS

### Scalability
- **Unlimited Tenants**: No PostgreSQL schema constraints
- **Consistent Performance**: Performance doesn't degrade with tenant count
- **Efficient Indexing**: Shared indexes across tenant data

### Maintenance
- **Single Schema**: Simplified backups and migrations
- **Unified Operations**: Single migration path for all tenants
- **Easier Monitoring**: One schema instead of hundreds

### Cost Efficiency
- **Shared Resources**: Connection pool used by all tenants
- **Reduced Overhead**: No per-tenant schema management
- **Better Utilization**: Efficient database resource usage

## Migration from Schema-per-tenant

If migrating from previous architecture:

1. **Data Consolidation**: Move data from multiple schemas to single `tenants` schema
2. **RLS Setup**: Implement Row-Level Security policies
3. **Connection Pool**: Simplify to single shared pool
4. **Query Updates**: Update queries for single schema with RLS context

## Troubleshooting

### Database Connection Issues
```bash
# Check Docker
docker ps && docker logs docker-db-1

# Test connectivity
curl http://localhost:3000/db/health

# Verify environment
cat .env | grep DB_
```

### RLS Issues
```bash
# Check RLS policies
SELECT schemaname, tablename, rowsecurity 
FROM pg_tables 
WHERE schemaname = 'tenants' AND rowsecurity = true;

# Check current RLS context
SELECT current_setting('rls.tenant_id', true);
```

This architecture provides better scalability, performance, and maintainability compared to schema-per-tenant approaches.