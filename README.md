# 🦀 Rust Axum Multi-Tenant API

A production-ready, multi-tenant REST API built with Rust and Axum, featuring comprehensive middleware layers, database multi-tenancy, and enterprise-grade architecture patterns.

## Overview

This API implements a sophisticated multi-tenant architecture with isolated database schemas, comprehensive middleware stack, and modular design patterns. Built for scalability, maintainability, and production deployment.

### ✨ Core API Features

1. **✅ Global Error Handling** - 404 for not found, 408 for timeouts, 413 for large payloads, 500 for internal errors
2. **✅ Unified Response Wrapper** - Consistent JSON output format for all endpoints
3. **✅ Structured Logging and Tracing** - Complete observability with `tracing` and `tracing-subscriber`
4. **✅ Graceful Shutdown** - Signal handling with proper resource cleanup
5. **✅ Hot Reload** - Development efficiency with `listenfd`, `systemfd`, and `cargo-watch`
6. **✅ Layered Environment Configuration** - Sophisticated `.env` file hierarchy
7. **✅ Modular Architecture** - Clear separation of concerns with feature-based modules
8. **✅ Multi-Tenant Database** - Schema-per-tenant isolation with PostgreSQL
9. **✅ Production Ready** - Comprehensive middleware stack and security features

### 🚀 Development Experience

- ✅ **Zero-setup development** - Ready-to-run with sensible defaults
- ✅ **Production-ready configuration** - Environment-based settings management
- ✅ **Consistent API responses** - Automatic formatting and error handling
- ✅ **Comprehensive error handling** - Proper HTTP status codes and error context
- ✅ **Hot reload for efficient development** - File watching with automatic recompilation
- ✅ **Graceful shutdown for reliable deployments** - Proper resource cleanup
- ✅ **Structured logging for observability** - Distributed tracing and performance monitoring

---

## API Architecture

### 1. Modular Structure

The API follows a domain-driven design with clear architectural boundaries:

```
src/
├── api/                    # API Layer - HTTP endpoints and routing
│   ├── test_api/          # Test endpoints for middleware validation
│   └── test_database/     # Database health and monitoring endpoints
├── config/                # Configuration Layer
│   ├── environment.rs     # Environment variable management with layered loading
│   └── state.rs          # Application state with singleton pattern
├── core/                  # Core Business Logic
│   ├── logging.rs        # Structured logging and tracing configuration
│   └── server.rs         # HTTP server setup and middleware stack
├── database/             # Data Layer
│   └── database_service.rs # Multi-tenant database service with connection pooling
└── utils/                # Cross-Cutting Concerns
    ├── error_handler/    # Global error handling middleware
    ├── response_handler/ # Unified response formatting system
    └── utils/           # Common utilities and helpers
```

### 2. Singleton Pattern Architecture

The application uses a centralized singleton pattern to ensure single instances of all critical components and shared resources:

#### AppState as the Only Singleton
```rust
#[derive(Debug, Clone)]
pub struct AppState {
    pub environment: Arc<EnvironmentVariables>,
    pub database: DatabaseService,
}

impl AppState {
    pub fn instance() -> &'static Self {
        static INSTANCE: Lazy<AppState> = Lazy::new(|| {
            AppState::new().expect("Failed to initialize AppState")
        });
        &INSTANCE
    }
}
```

#### Centralized Singleton Component Flow
```
AppState::instance() (Single Application Singleton)
├── environment: Arc<EnvironmentVariables>
│   └── Created once with EnvironmentVariables::load()
│   └── Shared safely across all components via Arc<T>
│   └── No independent singleton - only exists within AppState
└── database: DatabaseService
    └── Receives Arc<EnvironmentVariables> from AppState
    └── Contains HashMap<String, PgPool> for connection pools
    └── All database pools are singleton by containment
```

#### Key Benefits:
- **Single Source of Truth**: Only one singleton (`AppState`) manages all application state
- **Consistent Configuration**: One `EnvironmentVariables` instance shared across all components
- **Unique Database Pools**: Connection pools guaranteed to be application-wide singletons
- **Thread-Safe Sharing**: `Arc<EnvironmentVariables>` enables safe sharing across async tasks
- **Lazy Initialization**: All components initialize only when first accessed
- **Memory Efficiency**: No duplicate instances of heavy resources
- **No Singleton Conflicts**: Eliminates possibility of multiple configuration instances

#### Implementation Details:
```rust
// AppState creation flow - single instance creation
fn new() -> anyhow::Result<Self> {
    let environment = EnvironmentVariables::load()?;     // Create once
    let environment_arc = Arc::new(environment);         // Wrap for sharing
    Ok(Self {
        environment: environment_arc.clone(),            // Share same instance
        database: DatabaseService::new(environment_arc), // Share same config
    })
}
```

#### Access Pattern in Handlers:
```rust
async fn handler(State(app_state): State<AppState>) -> Result<Response, Error> {
    // Access singleton components through extracted app_state
    let db_service = &app_state.database;           // Singleton DatabaseService
    let config = &app_state.environment;           // Singleton EnvironmentVariables
    
    // All database operations use the same singleton pools
    let pool = db_service.get_tenant_pool("tenant_123", None).await?;
    
    // Access configuration values directly
    let timeout = config.default_timeout_seconds;
    let max_size = config.max_request_body_size;
}
```

### 3. Middleware Stack Architecture

The API implements a comprehensive middleware stack with specific error handling:

```rust
ServiceBuilder::new()
    .layer(from_fn(response_wrapper))           // Unified response formatting
    .layer(HandleErrorLayer::new(handle_global_error)) // Global error handling
    .layer(TimeoutLayer::new(Duration::from_secs(timeout))) // Request timeouts
    .layer(DefaultBodyLimit::max(max_size))     // Body size limits
```

#### Error Handling Matrix:
| Error Type | HTTP Status | Description |
|------------|------------|-------------|
| `LengthLimitError` | 413 Payload Too Large | Request body exceeds configured limit |
| `Elapsed` | 408 Request Timeout | Request exceeds configured timeout |
| `MatchedPathRejection` | 404 Not Found | Route not found in router |
| `InvalidUri` | 404 Not Found | Malformed URI patterns |
| Other | 500 Internal Server Error | Fallback for unhandled errors |

### 4. Response Standardization System

All API responses follow a consistent format through the `response_wrapper` middleware:

#### Standard Response Format:
```json
{
    "status": "OK",                    // HTTP status text
    "code": 200,                      // HTTP status code
    "data": { /* payload */ },        // Response data
    "messages": ["Success message"],   // Informational messages
    "date": "2025-06-03T18:03:14.523960+00:00"  // ISO timestamp
}
```

#### HandlerResponse Builder Pattern:
```rust
HandlerResponse::new(StatusCode::OK)
    .data(json!({ "user_id": 123, "name": "John" }))
    .message("User retrieved successfully")
```

### 5. Configuration Management

#### Layered Environment Loading:
The system implements a sophisticated configuration hierarchy:
```
System Environment Variables (highest priority)
    ↓
.env.production (production only)
    ↓  
.env.local (development overrides, gitignored)
    ↓
.env (base configuration, version controlled)
```

#### Configuration Validation:
- **Type Safety**: Automatic parsing with comprehensive error reporting
- **Missing Variables**: Aggregated error reporting for all missing required variables
- **Format Validation**: Specific validation for ports, URLs, and enum values
- **Environment Switching**: Automatic configuration loading based on `ENVIRONMENT` variable

#### Configuration Access Pattern:
All configuration access goes through the singleton `AppState`:
```rust
// Configuration is accessed via AppState singleton
async fn handler(State(app_state): State<AppState>) -> Result<Response, Error> {
    let config = &app_state.environment;
    
    // Access any configuration value
    let timeout = config.default_timeout_seconds;
    let max_size = config.max_request_body_size;
    let db_host = &config.db_host;
    let environment = &config.environment;
}
```

---

## API Capabilities

### 1. Database Management Endpoints

#### Health Check (`GET /db/health`)
- **Purpose**: Verifies database connectivity and health
- **Response**: Connection status with detailed error information
- **Use Case**: Load balancer health checks, monitoring systems

#### Monitoring (`GET /db/monitoring`)  
- **Purpose**: Provides database pool statistics and active connections
- **Response**: Active pools, connection counts, and performance metrics
- **Use Case**: Operational monitoring, capacity planning

### 2. Middleware Testing Endpoints

#### Hello Endpoint (`GET /hello`)
- **Purpose**: Basic connectivity test with version information
- **Features**: Demonstrates standard response format

#### Status Endpoint (`GET /status`)
- **Purpose**: API health check with environment information
- **Response**: API version, health status, environment details

#### Timeout Test (`GET /timeout`)
- **Purpose**: Tests timeout middleware behavior
- **Behavior**: Deliberately sleeps beyond configured timeout to trigger middleware

#### Error Test (`GET /error`)
- **Purpose**: Tests error handling middleware
- **Behavior**: Returns deliberate 500 error to validate error formatting

#### Body Size Test (`POST /body-size`)
- **Purpose**: Tests request body size limits
- **Features**: Processes and validates request body against configured limits

#### Not Found Test (`GET /not-found`)
- **Purpose**: Tests 404 error handling
- **Behavior**: Returns deliberate 404 to validate not found handling

---

## Technical Characteristics

### 1. Performance Optimization

#### Connection Pooling:
- **Per-tenant pools**: Isolated connection pools for data security
- **Pool sizing**: Configurable min/max connections (default: 5-20)
- **Idle timeout**: 30-second idle connection cleanup
- **Health monitoring**: Automatic pool health checks and recreation

#### Async Architecture:
- **Non-blocking I/O**: Full async/await implementation with Tokio
- **Concurrent requests**: Efficient request handling without thread blocking
- **Resource efficiency**: Minimal memory footprint with optimal resource utilization

### 2. Security Features

#### Multi-Tenant Isolation:
- **Schema-level separation**: Complete data isolation between tenants
- **Connection isolation**: Separate connection pools prevent data leakage
- **Search path configuration**: Automatic schema targeting for queries

#### SSL/TLS Configuration:
```rust
// Production: require SSL
options = options.ssl_mode(sqlx::postgres::PgSslMode::Require);

// Development: prefer SSL but don't require it
options = options.ssl_mode(sqlx::postgres::PgSslMode::Prefer);
```

#### Environment-based Security:
- **Production hardening**: Automatic SSL requirements and secure defaults
- **Development flexibility**: Relaxed settings for local development
- **Configuration validation**: Strict validation prevents misconfigurations

### 3. Observability & Monitoring

#### Structured Logging:
```rust
tracing::info!("Request processed", 
    user_id = %user_id, 
    tenant_id = %tenant_id,
    duration_ms = %duration.as_millis()
);
```

#### Distributed Tracing:
- **Span creation**: Automatic span creation for all endpoints
- **Backtrace capture**: Comprehensive error context in production
- **Performance tracking**: Request duration and resource usage monitoring

#### Health Metrics:
- **Database connectivity**: Real-time connection health monitoring
- **Pool statistics**: Active connections, idle connections, pool utilization
- **Error rates**: Automatic error classification and reporting

### 4. Operational Excellence

#### Graceful Shutdown:
```rust
tokio::select! {
    _ = ctrl_c => tracing::info!("Shutting down via Ctrl+C"),
    _ = terminate => tracing::info!("Shutting down via TERM signal"),
}
AppState::shutdown().await; // Closes all database connections
```

#### Hot Reload Support:
- **Development efficiency**: File watching with automatic recompilation
- **Zero-downtime updates**: Socket reuse for seamless development
- **Configuration reloading**: Dynamic environment variable updates

#### Docker Support:
- **Multi-stage builds**: Optimized production images
- **Development containers**: Isolated PostgreSQL for local development
- **Production deployment**: Ready for container orchestration platforms

---

## Multi-Tenant Database Service

The `DatabaseService` implements a schema-per-tenant pattern providing complete data isolation:

### 🗃️ Database Features

- ✅ **Automatic Initialization** - Master schema with `tenants` table on startup
- ✅ **UTC by Default** - All connections with UTC timezone configuration
- ✅ **Standard Fields** - `id`, `created_at`, `updated_at` with automatic triggers
- ✅ **Pool per Schema** - Independent connections (min: 5, max: 20)
- ✅ **Multi-app Support** - Multiple applications per tenant
- ✅ **Thread-safe** - `Arc<RwLock<HashMap<String, PgPool>>>`
- ✅ **SSL/TLS** - Automatic configuration based on environment
- ✅ **Graceful Shutdown** - Controlled connection closure

### Architecture Features:
- **Schema Isolation**: Each tenant gets a dedicated PostgreSQL schema
- **Connection Pooling**: Individual connection pools per tenant schema (max 20, min 5 connections)
- **Automatic Schema Management**: Dynamic schema creation and migration
- **UTC Standardization**: All timestamps in UTC with automatic triggers
- **SSL Configuration**: Environment-based SSL settings for security

### Key Components:
```rust
pub struct DatabaseService {
    /// Map of pool_key -> PgPool where key format is "{schema}_{app}" or just "{schema}"
    data_sources: Arc<RwLock<HashMap<String, PgPool>>>,
    /// Environment configuration
    config: Arc<EnvironmentVariables>,
}
```

### Standard Table Structure:
All tables follow a consistent pattern with automatic timestamp management:
```sql
CREATE TABLE example_table (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- custom fields here --
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Automatic Triggers:
Each table gets an automatic `updated_at` trigger for audit trail:
```sql
CREATE TRIGGER update_example_table_updated_at
    BEFORE UPDATE ON example_table
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### Database Schema Management

#### 1. Master Schema
The `master` schema contains the tenant registry:

```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_name VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### 2. Tenant Schemas
Each tenant gets a dedicated schema (`tenant_{id}`) with:
- **Isolated tables**: Complete data separation
- **Standard functions**: `update_updated_at_column()` for timestamp management
- **Automatic triggers**: `updated_at` field maintenance
- **UTC enforcement**: Timezone configuration for all connections

#### 3. Schema Operations
```rust
// Create tenant schema and get connection pool
let pool = database_service.get_tenant_pool("tenant_123", None).await?;

// Create table with standard timestamp fields
database_service.create_table_with_timestamps(
    &pool,
    "users",
    "email VARCHAR NOT NULL UNIQUE, name VARCHAR NOT NULL"
).await?;
```

---

## Deployment Architecture

### 1. Environment Configurations

#### Development:
```bash
ENVIRONMENT=development
HOST=127.0.0.1
PORT=3000
DB_HOST=localhost
DB_PORT=5432
MAX_REQUEST_BODY_SIZE=2097152  # 2MB
DEFAULT_TIMEOUT_SECONDS=30
```

#### Production:
```bash
ENVIRONMENT=production
HOST=0.0.0.0
PORT=3000
PROTOCOL=https
DB_HOST=production-db.amazonaws.com
DB_PORT=5432
DB_NAME=production_db
# SSL required automatically in production
```

### 2. Container Strategy

#### Local Development:
- **Application**: Native execution with hot reload
- **Database**: Docker container for isolation
- **Benefits**: Fast iteration, easy database recreation

#### Production:
- **Full containerization**: Multi-stage Docker builds
- **Optimized images**: Minimal production footprint
- **Platform ready**: AWS Fargate, ECS, Kubernetes compatible

### 3. Scalability Considerations

#### Horizontal Scaling:
- **Stateless design**: No session state, fully stateless architecture
- **Database pooling**: Per-instance connection management
- **Load balancer ready**: Health check endpoints for traffic routing

#### Vertical Scaling:
- **Resource efficiency**: Optimal memory and CPU utilization
- **Configurable limits**: Adjustable connection pools and timeouts
- **Performance monitoring**: Built-in metrics for capacity planning

---

## Dependencies & Technology Stack

### Core Framework:
- **Axum 0.8.4**: High-performance async web framework
- **Tokio**: Async runtime with full feature set
- **Tower**: Middleware and service composition
- **SQLx**: Async PostgreSQL driver with compile-time query checking

### Database & Storage:
- **PostgreSQL**: Multi-tenant database with schema isolation
- **Connection Pooling**: SQLx connection pool management
- **Migrations**: Built-in schema management and versioning

### Observability:
- **Tracing**: Structured logging and distributed tracing
- **Tracing-subscriber**: Log formatting and filtering
- **Chrono**: UTC timestamp management

### Configuration & Environment:
- **Dotenv**: Layered environment file loading
- **Once_cell**: Lazy static initialization for singletons
- **Anyhow**: Comprehensive error handling and context

### Development Tools:
- **Cargo-watch**: File watching for hot reload
- **Systemfd**: Socket reuse for zero-downtime reloads
- **Listenfd**: File descriptor passing for development

---

## API Testing & Validation

### Quick Health Check:
```bash
curl http://localhost:3000/hello
curl http://localhost:3000/db/health
```

### Middleware Validation:
```bash
# Test timeout (will take ~30+ seconds)
curl http://localhost:3000/timeout

# Test error handling
curl http://localhost:3000/error

# Test body size limits
echo '{"large": "payload"}' | curl -X POST http://localhost:3000/body-size -d @-
```

### Expected Response Format:
```json
{
    "status": "OK",
    "code": 200,
    "data": { "database": "connected" },
    "messages": ["Database connection healthy"],
    "date": "2025-06-03T18:03:14.523960+00:00"
}
```

---

## Benefits & Advantages

### 🏢 Enterprise-Grade Architecture
- **Multi-tenancy**: Complete data isolation with schema-per-tenant pattern
- **Scalability**: Horizontal and vertical scaling capabilities built-in
- **Security**: SSL/TLS, environment-based hardening, and connection isolation
- **Observability**: Comprehensive logging, tracing, and monitoring

### 🚀 Developer Experience
- **Zero-setup development**: Ready to run with Docker database
- **Hot reload**: Instant feedback during development
- **Type safety**: Compile-time guarantees with Rust and SQLx
- **Modular architecture**: Easy to extend and maintain

### 🔧 Production Ready
- **Graceful shutdown**: Proper resource cleanup and signal handling
- **Error handling**: Comprehensive middleware with proper HTTP status codes
- **Configuration management**: Environment-based settings with validation
- **Container ready**: Optimized builds for cloud deployment

### 💼 Business Value
- **Multi-tenant SaaS ready**: Complete tenant isolation and management
- **Compliance friendly**: Audit trails, UTC timestamps, and data separation
- **Cost effective**: Efficient resource utilization and connection pooling
- **Maintainable**: Clear separation of concerns and documentation

This API provides a robust foundation for building scalable, multi-tenant applications with enterprise-grade reliability, security, and observability features built-in from day one. 