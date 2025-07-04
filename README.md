# IC360 API - Rust Axum Multi-Tenant Service

High-performance REST API built with Rust and Axum, featuring multi-tenant architecture with Row-Level Security.

## 🚀 Quick Start

```bash
# 1. Start PostgreSQL
docker-compose -f docker/docker-compose.dev.yml up -d

# 2. Setup environment
cp .env.example .env

# 3. Run API
cargo run
```

The API will be available at `http://localhost:3000`

## ✨ Core API Features

✅ **Global Error Handling** - 404 for not found, 408 for timeouts, 413 for large payloads, 500 for internal errors
✅ **Unified Response Wrapper** - Consistent JSON output format for all endpoints
✅ **Best Practices** - Singleton pattern for env variables and db connection pools
✅ **Structured Logging and Tracing** - Complete observability with tracing and tracing-subscriber
✅ **Graceful Shutdown** - Signal handling with proper resource cleanup
✅ **Hot Reload** - Development efficiency with listenfd, systemfd, and cargo-watch
✅ **Layered Environment Configuration** - Sophisticated .env file hierarchy
✅ **Modular Architecture** - Clear separation of concerns with feature-based modules
✅ **Multi-Tenant Database** - Single schema with Row-Level Security using PostgreSQL and SQLx
✅ **Production Ready** - Comprehensive middleware stack and security features

## 🚀 Development Experience

✅ **Zero-setup development** - Ready-to-run with sensible defaults
✅ **Production-ready configuration** - Environment-based settings management
✅ **Consistent API responses** - Automatic formatting and error handling
✅ **Comprehensive error handling** - Proper HTTP status codes and error context
✅ **Hot reload for efficient development** - File watching with automatic recompilation
✅ **Graceful shutdown for reliable deployments** - Proper resource cleanup
✅ **Structured logging for observability** - Distributed tracing and performance monitoring

## 🏗️ Architecture

```
src/
├── api/                    # HTTP endpoints and routing
│   ├── tenants/           # Tenant management endpoints
│   ├── test_api/          # Test endpoints for middleware validation
│   └── test_database/     # Database health and monitoring
├── config/                # Configuration and environment management
├── core/                  # Server setup and logging
├── database/              # Database service with RLS
└── utils/                 # Response/error handling and utilities
```

## 🎯 Core Endpoints

### Tenants
- `POST /tenants` - Create new tenant
- `GET /tenants` - List all tenants

### Health & Monitoring
- `GET /hello` - Basic health check
- `GET /db/health` - Database connectivity check
- `GET /db/monitoring` - Database statistics
- `GET /db/tenants/monitoring` - Tenant-specific metrics

### Testing
- `GET /status` - System status with environment info
- `GET /error` - Test error handling
- `POST /body-size` - Test request body limits

## 🗃️ Database Architecture

**Single Schema with Row-Level Security:**
- All tenants share the `tenants` schema for optimal performance
- PostgreSQL RLS policies provide data isolation
- Single connection pool for efficient resource usage
- Unlimited tenant scalability

### Benefits
- **Scalability**: No PostgreSQL schema limit constraints
- **Performance**: Consistent performance regardless of tenant count
- **Maintenance**: Simplified backups and migrations
- **Cost**: Efficient resource utilization

## ⚙️ Configuration

### Environment Files
```
.env.production (production overrides)
.env.local (development overrides, gitignored)
.env (base configuration)
```

### Key Variables
```env
# Server
HOST=localhost
PORT=3000
MAX_REQUEST_BODY_SIZE=2097152
DEFAULT_TIMEOUT_SECONDS=30

# Database
DB_HOST=localhost
DB_PORT=5432
DB_NAME=rust_axum_api
DB_USER=postgres
DB_PASSWORD=postgres
```

## 🐳 Docker Setup

### Development
```bash
docker-compose -f docker/docker-compose.dev.yml up -d
```

### Production
```bash
docker build -f docker/Dockerfile.production -t ic360-api .
docker run -p 3000:3000 ic360-api
```

## 🧪 Testing

```bash
# Health check
curl http://localhost:3000/hello

# Create tenant
curl -X POST http://localhost:3000/tenants \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "test_company"}'

# Monitor database
curl http://localhost:3000/db/tenants/monitoring
```

## 📦 Dependencies

- **Axum 0.8** - HTTP framework
- **SQLx** - PostgreSQL driver with compile-time checking
- **Tokio** - Async runtime
- **Tower** - Middleware framework
- **Tracing** - Structured logging

## 🔧 Development

### Hot Reload
```bash
cargo install cargo-watch systemfd
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

### Environment
- Development: Debug logging, relaxed SSL
- Production: Error-only logging, required SSL

---

Built with ❤️ using Rust and Axum 