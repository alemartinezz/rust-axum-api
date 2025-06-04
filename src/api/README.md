# 🚀 API Module

This module contains all HTTP endpoints and API functionality for the multi-tenant Rust Axum application. It provides a clean separation of concerns with dedicated handlers, routes, and request/response models.

## 📂 Module Structure

```
src/api/
├── mod.rs                    # Module exports
├── test_api/                 # API testing endpoints
│   ├── handler.rs           # Test API handlers
│   ├── routes.rs            # Test API route definitions
│   └── mod.rs               # Module exports
├── test_database/           # Database testing endpoints
│   ├── handler.rs           # Database test handlers
│   ├── routes.rs            # Database test routes
│   └── mod.rs               # Module exports
└── tenants/                 # Tenant management endpoints
    ├── handler.rs           # Tenant CRUD handlers
    ├── routes.rs            # Tenant route definitions
    └── mod.rs               # Module exports
```

## 🔌 Available Endpoints

### **Test API Endpoints (`/test-api`)**
- **`GET /test-api/health`** - Basic health check endpoint
- **`GET /test-api/hello`** - Simple hello world endpoint
- **`POST /test-api/echo`** - Echo back request data
- **`GET /test-api/error`** - Test error handling
- **`GET /test-api/timeout`** - Test timeout handling

### **Database Test Endpoints (`/db`)**
- **`GET /db/health`** - Database connectivity check
- **`GET /db/monitoring`** - Active connection pools monitoring

### **Tenant Management Endpoints (`/tenants`)**
- **`POST /tenants`** - Create new tenant with isolated schema
- **`GET /tenants`** - List all existing tenants
- **`GET /tenants/{id}`** - Get specific tenant details
- **`PUT /tenants/{id}`** - Update tenant information
- **`DELETE /tenants/{id}`** - Delete tenant and cleanup schema

## 🏗️ Architecture Patterns

### **Handler-Route Separation**
Each endpoint group follows a consistent pattern:
- **Routes**: Define URL patterns and HTTP methods
- **Handlers**: Contain business logic and data processing
- **Models**: Request/response structures with validation

### **State Management**
All handlers receive the application state via Axum's `State` extractor:
```rust
pub async fn handler(State(state): State<AppState>) -> HandlerResponse {
    // Access to database, environment variables, etc.
    let db_service = &state.database;
    let env = &state.environment;
}
```

### **Response Standardization**
All endpoints use the unified `HandlerResponse` system:
```rust
HandlerResponse::new(StatusCode::OK)
    .data(json!(response_data))
    .message("Operation completed successfully")
```

## 🎯 Tenant Management Features

### **Multi-Schema Isolation**
- Each tenant gets a dedicated PostgreSQL schema (`tenant_{uuid}`)
- Complete data isolation between tenants
- Automatic schema creation and initialization
- Proper cleanup on tenant deletion

### **Tenant Creation Process**
1. **Validation**: Name format, length, uniqueness checks
2. **UUID Generation**: Unique identifier for tenant
3. **Master Registration**: Record in master tenants table
4. **Schema Creation**: Dedicated schema with standard tables
5. **Pool Initialization**: Connection pool for tenant schema

### **Tenant Data Structure**
```json
{
  "id": "uuid-v4",
  "tenant_name": "customer_name",
  "schema_name": "tenant_abc123",
  "created_at": "2024-01-01T00:00:00Z"
}
```

## 🔧 Testing Endpoints

### **API Health Checks**
- **Basic connectivity**: Server responsiveness
- **Middleware validation**: Error handling, timeouts, body limits
- **Response formatting**: Unified JSON structure

### **Database Monitoring**
- **Connection health**: PostgreSQL connectivity
- **Pool statistics**: Active connections per tenant
- **Schema validation**: Table existence and structure

### **Error Simulation**
- **Timeout testing**: Long-running operation simulation
- **Error handling**: Exception propagation testing
- **Status code validation**: HTTP response correctness

## 📊 Request/Response Models

### **Create Tenant Request**
```rust
#[derive(Deserialize)]
pub struct CreateTenantRequest {
    pub tenant_name: String,  // 1-100 chars, alphanumeric + _-
}
```

### **Standard Response Format**
```json
{
  "status": "CREATED",
  "code": 201,
  "data": { /* response payload */ },
  "messages": ["Tenant created successfully"],
  "date": "2024-01-01T00:00:00Z"
}
```

## 🛡️ Validation & Security

### **Input Validation**
- **Tenant names**: Length limits, character restrictions
- **Request payloads**: Size limits via middleware
- **SQL injection**: Parameterized queries only

### **Error Handling**
- **Graceful degradation**: Proper error responses
- **Information hiding**: No internal details in production
- **Logging**: Structured error tracking

### **Resource Protection**
- **Connection pooling**: Prevents connection exhaustion
- **Timeout enforcement**: Prevents hanging requests
- **Memory limits**: Request body size restrictions

## 🚀 Usage Examples

### **Creating a New Tenant**
```bash
curl -X POST http://localhost:3000/tenants \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "acme_corp"}'
```

### **Listing Tenants**
```bash
curl http://localhost:3000/tenants
```

### **Health Check**
```bash
curl http://localhost:3000/test-api/health
```

### **Database Monitoring**
```bash
curl http://localhost:3000/db/monitoring
```

## 🔄 Integration with Core Systems

### **Database Service**
- **Schema management**: Automatic creation and cleanup
- **Connection pooling**: Efficient resource utilization
- **Transaction handling**: ACID compliance for tenant operations

### **Response Handler**
- **Unified formatting**: Consistent JSON responses
- **Automatic wrapping**: Middleware-based response transformation
- **Logging integration**: Request/response tracking

### **Error Handler**
- **Global error catching**: Centralized error processing
- **Status code mapping**: HTTP-appropriate error responses
- **Graceful degradation**: Service continuity

## 📈 Performance Considerations

### **Connection Pool Efficiency**
- **Pool per tenant**: Isolated but efficient connections
- **Connection reuse**: Minimize TCP overhead
- **Idle timeout**: Resource cleanup

### **Request Processing**
- **Async handlers**: Non-blocking request processing
- **Structured logging**: Performance tracking
- **Timeout enforcement**: Prevent resource starvation

### **Scalability Patterns**
- **Stateless design**: Horizontal scaling ready
- **Database isolation**: Independent tenant scaling
- **Resource limits**: Prevent tenant interference

## 🧪 Testing Strategy

### **Unit Tests**
- Handler logic validation
- Input validation testing
- Error condition coverage

### **Integration Tests**
- End-to-end request flow
- Database interaction testing
- Middleware behavior validation

### **Load Testing**
- Concurrent tenant creation
- Connection pool stress testing
- Response time validation

This API module provides a robust, scalable foundation for multi-tenant applications with comprehensive testing, monitoring, and management capabilities. 