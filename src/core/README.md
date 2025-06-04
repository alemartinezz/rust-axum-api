# 🎯 Core Module

This module contains the fundamental infrastructure components of the application: server configuration, logging system, and core application lifecycle management. It provides the foundation upon which all other modules operate.

## 📂 Module Structure

```
src/core/
├── mod.rs              # Module exports
├── server.rs           # Server setup and middleware configuration
└── logging.rs          # Structured logging and tracing system
```

## 🖥️ Server Management (`server.rs`)

### **Application Router Creation**
The core server module provides a sophisticated, layered approach to building the HTTP server with comprehensive middleware stack:

```rust
pub fn create_app() -> Router {
    Router::new()
        .merge(test_api_routes())
        .merge(test_database::test_database_routes())
        .merge(tenant_routes())
        .layer(/* comprehensive middleware stack */)
        .with_state(AppState::instance().clone())
}
```

### **Middleware Stack Architecture**
The middleware layers are applied in a specific order for optimal request processing:

```rust
ServiceBuilder::new()
    .layer(from_fn(response_wrapper))           // 1. Response standardization
    .layer(HandleErrorLayer::new(handle_global_error))  // 2. Error handling
    .layer(TimeoutLayer::new(Duration::from_secs(timeout)))  // 3. Request timeouts
    .layer(DefaultBodyLimit::max(max_size))     // 4. Body size limits
```

### **Middleware Layer Details**

#### **1. Response Wrapper**
- **Purpose**: Standardizes all HTTP responses to unified JSON format
- **Location**: Outermost layer (processes responses on the way out)
- **Functionality**: Wraps raw responses in `ResponseFormat` structure

#### **2. Global Error Handler**
- **Purpose**: Catches and maps errors to appropriate HTTP status codes
- **Handles**: Timeouts (408), payload too large (413), not found (404), internal errors (500)
- **Integration**: Works with Tower's error handling infrastructure

#### **3. Timeout Layer**
- **Purpose**: Prevents requests from hanging indefinitely
- **Configuration**: Configurable via `DEFAULT_TIMEOUT_SECONDS` environment variable
- **Behavior**: Automatically cancels requests exceeding timeout limit

#### **4. Body Limit Layer**
- **Purpose**: Protects against oversized request payloads
- **Configuration**: Configurable via `MAX_REQUEST_BODY_SIZE` environment variable
- **Security**: Prevents memory exhaustion attacks

### **TCP Listener Setup**
Supports both hot reload and standard deployment scenarios:

```rust
pub async fn setup_listener() -> Result<TcpListener> {
    let mut listenfd = ListenFd::from_env();
    
    // Hot reload support (development)
    match listenfd.take_tcp_listener(0)? {
        Some(std_listener) => {
            // Reuse existing listener for hot reload
            TcpListener::from_std(std_listener)?
        }
        None => {
            // Create new listener for normal startup
            let addr = format!("{}:{}", env.host, env.port);
            TcpListener::bind(&addr).await?
        }
    }
}
```

### **Graceful Shutdown System**
Comprehensive signal handling for clean application termination:

```rust
pub async fn shutdown_signal() {
    tokio::select! {
        _ = ctrl_c => tracing::info!("Shutting down via Ctrl+C"),
        _ = terminate => tracing::info!("Shutting down via TERM signal"),
    }
    
    // Graceful cleanup
    AppState::shutdown().await;
}
```

## 📊 Logging System (`logging.rs`)

### **Environment-Aware Configuration**
The logging system adapts to different deployment environments automatically:

| Environment | Application Logs | Framework Logs | SQLx Logs | Span Events |
|-------------|------------------|----------------|-----------|-------------|
| **Development** | INFO | WARN | WARN | NONE |
| **Production** | INFO | ERROR | ERROR | NONE |
| **Debug** | DEBUG | DEBUG | INFO | CLOSE |

### **Logging Configuration Details**

#### **Development Environment (default)**
```rust
"my_axum_project=info,sqlx=warn,tower_http=warn,axum=warn"
```
- **Application logs**: Informational and above
- **Framework logs**: Warnings only (reduces noise)
- **Database logs**: Warnings only (avoids query spam)
- **Span events**: Disabled for cleaner output

#### **Production Environment**
```rust
"my_axum_project=info,sqlx=error,tower_http=error,axum=error"
```
- **Application logs**: Important information only
- **Framework logs**: Errors only (minimal noise)
- **Database logs**: Errors only (critical issues only)
- **Focus**: Performance and critical issue detection

#### **Debug Environment**
```rust
"my_axum_project=debug,sqlx=info,tower_http=debug,axum=debug"
```
- **Application logs**: Verbose debugging information
- **Framework logs**: Full debugging details
- **Database logs**: Connection and query information
- **Span events**: Shows timing information (enter/exit)

### **Structured Logging Features**
- **Hierarchical Filtering**: Different log levels per component
- **Environment Override**: `RUST_LOG` environment variable support
- **Span Tracking**: Request lifecycle tracking in debug mode
- **Performance Monitoring**: Timing information for operations

### **Manual Override Support**
```bash
# Override for specific debugging
RUST_LOG=my_axum_project=debug,sqlx=info cargo run

# Trace specific components
RUST_LOG=tower_http=trace,axum=debug cargo run
```

## 🔄 Application Lifecycle

### **Startup Sequence**
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize logging system
    logging::init_tracing();
    
    // 2. Initialize database and master schema
    AppState::init_master_schema().await?;
    
    // 3. Create application router
    let app = server::create_app();
    
    // 4. Setup TCP listener
    let listener = server::setup_listener().await?;
    
    // 5. Start server with graceful shutdown
    serve(listener, app)
        .with_graceful_shutdown(server::shutdown_signal())
        .await?;
}
```

### **Shutdown Sequence**
1. **Signal Reception**: Ctrl+C or TERM signal received
2. **Connection Draining**: Existing requests allowed to complete
3. **Resource Cleanup**: Database connections closed gracefully
4. **Application Exit**: Clean termination with proper status codes

## 🛡️ Error Handling Architecture

### **Error Type Mapping**
The global error handler provides intelligent error type detection and mapping:

```rust
// Body size exceeded
LengthLimitError → 413 PAYLOAD_TOO_LARGE

// Request timeout
Elapsed → 408 REQUEST_TIMEOUT

// Route not found
MatchedPathRejection → 404 NOT_FOUND

// All other errors
_ → 500 INTERNAL_SERVER_ERROR
```

### **Error Chain Traversal**
Sophisticated error cause detection using source chain traversal:
```rust
pub fn find_cause<T: Error + 'static>(err: &dyn Error) -> Option<&T> {
    let mut source = err.source();
    while let Some(s) = source {
        if let Some(typed) = s.downcast_ref::<T>() {
            return Some(typed);
        }
        source = s.source();
    }
    None
}
```

## 🚀 Hot Reload Support

### **Development Workflow**
1. **File Watching**: `cargo-watch` monitors source files
2. **Graceful Handoff**: `listenfd` preserves TCP listener
3. **Zero Downtime**: Connections maintained during restart
4. **State Reinitialization**: Fresh application state on restart

### **Hot Reload Command**
```bash
# Development with hot reload
systemfd --no-pid -s http::3000 -- cargo watch -x run

# Manual restart maintains connections
kill -USR1 <server_pid>
```

## ⚡ Performance Optimizations

### **Middleware Ordering**
Middleware is ordered for optimal performance:
1. **Body limits** (early rejection of oversized requests)
2. **Timeouts** (prevent resource starvation)
3. **Error handling** (centralized error processing)
4. **Response wrapping** (standardization on successful responses)

### **Connection Management**
- **Non-blocking I/O**: Full async/await throughout
- **Connection reuse**: TCP listener reuse for hot reload
- **Resource pooling**: Database connection pooling
- **Graceful degradation**: Proper cleanup on shutdown

### **Memory Efficiency**
- **Arc sharing**: State shared via reference counting
- **Lazy initialization**: Components created on-demand
- **Efficient logging**: Environment-appropriate log levels

## 🧪 Testing Infrastructure

### **Unit Testing Support**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_server_creation() {
        let app = create_app();
        // Test router configuration
    }
}
```

### **Integration Testing**
- **Test harness**: Configurable test server setup
- **Mock state**: Test-specific application state
- **Request simulation**: Full HTTP request/response testing

## 🏗️ Extensibility Patterns

### **Middleware Extension**
```rust
// Adding new middleware layers
Router::new()
    .merge(existing_routes())
    .layer(
        ServiceBuilder::new()
            .layer(custom_middleware)      // Add here
            .layer(existing_middleware)
            // ... rest of stack
    )
```

### **Route Integration**
```rust
// Adding new route modules
Router::new()
    .merge(test_api_routes())
    .merge(tenant_routes())
    .merge(new_feature_routes())        // Add here
    .layer(middleware_stack)
```

### **Logging Extension**
```rust
// Custom log levels per module
"my_axum_project=info,new_module=debug,sqlx=warn"
```

## 📈 Monitoring & Observability

### **Structured Logging Output**
```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "INFO",
  "target": "my_axum_project::api::tenants",
  "message": "Creating new tenant: acme_corp",
  "fields": {
    "tenant_name": "acme_corp"
  }
}
```

### **Request Tracing**
- **Span tracking**: Request lifecycle monitoring
- **Performance metrics**: Response time measurement
- **Error correlation**: Error tracking across request lifecycle
- **Resource monitoring**: Database connection usage

## 🏆 Production Readiness

### **Deployment Features**
- **Signal handling**: Proper UNIX signal support
- **Graceful shutdown**: Clean resource cleanup
- **Error resilience**: Comprehensive error handling
- **Performance monitoring**: Structured logging and metrics

### **Security Considerations**
- **Request limits**: Body size and timeout protection
- **Error information**: Production-safe error responses
- **Resource protection**: Connection and memory limits
- **Signal security**: Safe signal handling practices

This core module provides the robust foundation necessary for enterprise-grade web applications with comprehensive infrastructure, monitoring, and lifecycle management capabilities. 