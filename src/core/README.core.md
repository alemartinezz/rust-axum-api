# Core Module

Contains the fundamental infrastructure components: server setup, logging system, and application lifecycle management.

## Structure

```
src/core/
├── mod.rs              # Module exports
├── server.rs           # Server setup and middleware
└── logging.rs          # Structured logging and tracing
```

## Server Management

### Application Router
```rust
pub fn create_app() -> Router {
    Router::new()
        .merge(test_api_routes())
        .merge(test_database::test_database_routes())
        .merge(tenant_routes())
        .layer(/* middleware stack */)
        .with_state(AppState::instance().clone())
}
```

### Middleware Stack
Applied in specific order for optimal request processing:

```rust
ServiceBuilder::new()
    .layer(from_fn(response_wrapper))           // Response standardization
    .layer(HandleErrorLayer::new(handle_global_error))  // Error handling
    .layer(TimeoutLayer::new(Duration::from_secs(timeout)))  // Request timeouts
    .layer(DefaultBodyLimit::max(max_size))     // Body size limits
```

### Middleware Details
- **Response Wrapper**: Standardizes all responses to unified JSON format
- **Error Handler**: Maps errors to appropriate HTTP status codes (408, 413, 404, 500)
- **Timeout Layer**: Prevents hanging requests with configurable timeout
- **Body Limit**: Protects against oversized payloads

### TCP Listener Setup
Supports both hot reload and standard deployment:

```rust
pub async fn setup_listener() -> Result<TcpListener> {
    let mut listenfd = ListenFd::from_env();
    
    match listenfd.take_tcp_listener(0)? {
        Some(std_listener) => TcpListener::from_std(std_listener)?, // Hot reload
        None => {
            let addr = format!("{}:{}", env.host, env.port);
            TcpListener::bind(&addr).await?  // Normal startup
        }
    }
}
```

### Graceful Shutdown
```rust
pub async fn shutdown_signal() {
    tokio::select! {
        _ = ctrl_c => info!("Shutting down via Ctrl+C"),
        _ = terminate => info!("Shutting down via TERM signal"),
    }
    AppState::shutdown().await;
}
```

## Logging System

### Environment-Based Configuration

| Environment | App Logs | Framework | SQLx | Spans |
|-------------|----------|-----------|------|-------|
| **Development** | INFO | WARN | WARN | NONE |
| **Production** | INFO | ERROR | ERROR | NONE |
| **Debug** | DEBUG | DEBUG | INFO | CLOSE |

### Log Level Details

#### Development (default)
```rust
"my_axum_project=info,sqlx=warn,tower_http=warn,axum=warn"
```
- Informational app logs, warnings only for framework
- Reduced noise for development productivity

#### Production
```rust
"my_axum_project=info,sqlx=error,tower_http=error,axum=error"
```
- Minimal logging for performance
- Errors only for critical issues

#### Debug
```rust
"my_axum_project=debug,sqlx=info,tower_http=debug,axum=debug"
```
- Verbose debugging with timing information
- Shows request lifecycle and database queries

### Manual Override
```bash
# Override for specific debugging
RUST_LOG=my_axum_project=debug,sqlx=info cargo run

# Trace HTTP requests
RUST_LOG=tower_http=trace,axum=debug cargo run
```

## Application Lifecycle

### Startup Sequence
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::init_tracing();                    // 1. Initialize logging
    AppState::init_master_schema().await?;     // 2. Setup database
    let app = server::create_app();            // 3. Create router
    let listener = server::setup_listener().await?;  // 4. Setup TCP listener
    
    serve(listener, app)
        .with_graceful_shutdown(server::shutdown_signal())  // 5. Start with shutdown
        .await?;
}
```

### Shutdown Sequence
1. **Signal Reception**: Ctrl+C or TERM signal
2. **Connection Draining**: Complete existing requests
3. **Resource Cleanup**: Close database connections
4. **Clean Exit**: Proper status codes

## Error Handling

### Error Type Mapping
```rust
// Request body too large
LengthLimitError → 413 PAYLOAD_TOO_LARGE

// Request timeout
Elapsed → 408 REQUEST_TIMEOUT

// Route not found
MatchedPathRejection → 404 NOT_FOUND

// All others
Default → 500 INTERNAL_SERVER_ERROR
```

## Features

- **Hot Reload**: Development efficiency with `listenfd` and `systemfd`
- **Graceful Shutdown**: Proper signal handling for deployments
- **Structured Logging**: Environment-aware log configuration
- **Middleware Stack**: Comprehensive request/response processing
- **Error Mapping**: Intelligent error type detection and HTTP mapping
- **Performance**: Optimized for both development and production 