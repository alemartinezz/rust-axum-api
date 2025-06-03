# Database Multi-Tenancy Module

Multi-tenant database system using SQLx and PostgreSQL with automatic initialization, UTC defaults, and standard field patterns.

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────────────────────────┐
│   AppState      │    │         DatabaseService             │
│                 │───▶│                                     │
│ - environment   │    │ ┌─────────────────────────────────┐ │
│ - database      │    │ │     HashMap<String, PgPool>     │ │
└─────────────────┘    │ │                                 │ │
                       │ │ "master" ──▶ PgPool             │ │
                       │ │ "tenant_123" ──▶ PgPool         │ │
                       │ │ "tenant_123_web" ──▶ PgPool     │ │
                       │ │ "tenant_123_mobile" ──▶ PgPool  │ │
                       │ └─────────────────────────────────┘ │
                       └─────────────────────────────────────┘
```

**Schema-per-Tenant Pattern using PostgreSQL schemas:**

```
PostgreSQL Database (timezone: UTC)
├── Schema: master          (tenants registry table)
├── Schema: tenant_123      (tenant 123 tables)
├── Schema: tenant_456      (tenant 456 tables)
└── ...
```

## ✨ Key Features

- ✅ **Automatic Initialization** - Master schema with `tenants` table on startup
- ✅ **UTC by Default** - All connections with UTC timezone configuration
- ✅ **Standard Fields** - `id`, `created_at`, `updated_at` with automatic triggers
- ✅ **Pool per Schema** - Independent connections (min: 5, max: 20)
- ✅ **Multi-app Support** - Multiple applications per tenant
- ✅ **Thread-safe** - `Arc<RwLock<HashMap<String, PgPool>>>`
- ✅ **SSL/TLS** - Automatic configuration based on environment
- ✅ **Graceful Shutdown** - Controlled connection closure

## 🚀 Quick Start

### 1. Environment Configuration
```env
ENVIRONMENT=development
DB_HOST=localhost
DB_PORT=5432
DB_NAME=rust_axum_api
DB_USER=postgres
DB_PASSWORD=postgres
```

### 2. Start PostgreSQL with Docker
```bash
docker-compose -f docker/docker-compose.dev.yml up -d
```

### 3. Run Application
```bash
cargo run
# ✅ Automatic master schema and tenants table initialization
```

## 💻 Usage

### Access DatabaseService
```rust
use crate::config::state::AppState;

async fn my_handler(State(app_state): State<AppState>) -> Result<(), Error> {
    let db_service = &app_state.database;
    
    // Master schema (tenants table)
    let master_pool = db_service.get_master_pool().await?;
    
    // Tenant schema (created automatically)
    let tenant_pool = db_service.get_tenant_pool("tenant_123", None).await?;
    
    Ok(())
}
```

### Create Tables with Standard Fields
```rust
// Helper automatically creates: id, created_at, updated_at + triggers
db_service.create_table_with_timestamps(
    &tenant_pool,
    "users",
    "email VARCHAR NOT NULL UNIQUE, name VARCHAR NOT NULL"
).await?;
```

### Programmatic Tenant Creation
```rust
async fn create_new_tenant(db: &DatabaseService, name: &str) -> Result<String> {
    let tenant_id = Uuid::new_v4().simple().to_string();
    
    // 1. Register in master.tenants
    let master_pool = db.get_master_pool().await?;
    sqlx::query("INSERT INTO tenants (tenant_name) VALUES ($1)")
        .bind(name)
        .execute(&master_pool)
        .await?;
    
    // 2. Create tenant schema
    let _tenant_pool = db.get_tenant_pool(&tenant_id, None).await?;
    
    Ok(tenant_id)
}
```

## 📊 Standard Table Structure

All tables follow this pattern:

```sql
CREATE TABLE example_table (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- table-specific fields --
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Automatic trigger for updated_at
CREATE TRIGGER update_example_table_updated_at
    BEFORE UPDATE ON example_table
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

## 🔧 Available APIs

```bash
# Health check
curl http://localhost:3000/db/health

# Active pools monitoring
curl http://localhost:3000/db/monitoring
```

## 🌍 UTC Configuration

- **UTC Timezone**: Automatically configured on all connections
- **TIMESTAMPTZ**: Standard data type for dates
- **NOW()**: UTC function in automatic triggers

## 🏢 Multi-App Support

```rust
// Pool for tenant's web app
let web_pool = db_service.get_tenant_pool("tenant_123", Some("web")).await?;

// Pool for same tenant's mobile app
let mobile_pool = db_service.get_tenant_pool("tenant_123", Some("mobile")).await?;
```

## 📈 Monitoring & Management

```rust
// List active pools
let active_pools = db_service.list_active_pools().await;

// Specific pool statistics
let (total, idle) = db_service.get_pool_stats("tenant_123", None).await?;

// Close specific pool
db_service.close_pool("tenant_123", Some("web")).await?;
```

## 🔒 Security & Performance

- **Complete Isolation**: Each tenant in its own schema
- **Independent Pools**: Connection management per schema
- **Automatic Escaping**: Schema names properly escaped
- **SSL/TLS**: Automatic prod/dev configuration
- **Pool Caching**: Automatic pool reuse

## 🛠️ Troubleshooting

### Database doesn't exist
```bash
# Check Docker
docker ps
docker logs docker-db-1

# Verify .env
cat .env | grep DB_NAME
```

### Connection issues
```bash
# Test connectivity
curl http://localhost:3000/db/health

# Monitor pools
curl http://localhost:3000/db/monitoring
```

## 📁 Module Structure

```
src/database/
├── mod.rs                  # Exports DatabaseService
├── database_service.rs     # Main implementation
└── README.md              # This documentation
```

The system is designed to be **completely automatic**: startup initialization, programmatic tenant creation, and transparent management of UTC connections with standard fields. 