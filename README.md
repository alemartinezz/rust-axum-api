# 🦀 Rust Axum API Template - Comprehensive Guide

## Introduction

This guide will help you understand and extend this Axum-based API template that provides:

1. **✅ Global Error Handling** (404 for not found, 408 for timeouts, 413 for large payloads, 500 for internal errors)
2. **✅ Unified Response Wrapper** for consistent JSON output
3. **✅ Structured Logging and Tracing** (via `tracing`, `tracing-subscriber`)
4. **✅ Graceful Shutdown** with signal handling
5. **✅ Hot Reload** in development (using `listenfd`, `systemfd`, and `cargo-watch`)
6. **✅ Layered Environment Configuration** with `.env` support
7. **✅ Modular Architecture** with clear separation of concerns

## Project Structure

The template follows a clean, modular architecture:

```bash
rust-axum-api/
├── Cargo.toml
├── .env                    # Base configuration (included in repo)
├── .env.local.example      # Local override example
├── src/
│   ├── api/               # API endpoints and routes
│   │   └── test/          # Test endpoints for middleware validation
│   │       ├── handler.rs
│   │       ├── mod.rs
│   │       └── routes.rs
│   ├── core/              # Core business logic
│   │   ├── logging.rs     # Tracing configuration
│   │   ├── server.rs      # Server setup and configuration
│   │   └── mod.rs
│   ├── config/            # Application configuration
│   │   ├── environment.rs # Environment variables management
│   │   ├── state.rs       # Application state
│   │   └── mod.rs
│   ├── utils/             # Shared utilities
│   │   ├── error_handler/ # Global error handling
│   │   ├── response_handler/ # Response formatting
│   │   ├── utils/         # Common utilities
│   │   └── mod.rs
│   ├── lib.rs
│   └── main.rs            # Application entry point
```

---

## 1. Quick Start

### 1.1. Clone and Run

```bash
git clone <your-repo>
cd rust-axum-api
cargo run
```

**That's it!** The project includes a ready-to-use `.env` file with sensible defaults.

### 1.2. Test the API

The API includes several test endpoints to validate different middleware components:

```bash
# Basic hello endpoint
curl http://127.0.0.1:3000/hello

# API status endpoint
curl http://127.0.0.1:3000/test/status

# Test timeout middleware (will take ~4 seconds)
curl http://127.0.0.1:3000/test/timeout

# Test error handling
curl http://127.0.0.1:3000/test/error

# Test not found handling
curl http://127.0.0.1:3000/test/not-found

# Test body size limits with small payload
curl -X POST http://127.0.0.1:3000/test/body-size \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'
```

Expected response format for successful requests:
```json
{
  "status": "OK",
  "code": 200,
  "data": {
    "version": "1.0.0"
  },
  "messages": [
    "Service started successfully"
  ],
  "date": "2025-06-02T19:58:06.486652+00:00"
}
```

---

## 2. Architecture Overview

### 2.1. Main Entry Point (`src/main.rs`)

The main function is simplified and focused on orchestration:

```rust
// The main entry point - simplified and focused on orchestration only

use axum::serve;

use my_axum_project::config::{state::AppState, environment::EnvironmentVariables};
use my_axum_project::core::{logging, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    logging::init_tracing();

    // Initialize environment variables
    let _ = EnvironmentVariables::instance();
    
    // Create the application state
    let state: AppState = AppState::new();
    
    // Setup the application router
    let app: axum::Router = server::create_app(state);
    
    // Setup the TCP listener
    let listener: tokio::net::TcpListener = server::setup_listener().await?;

    // Show listening address
    println!("Server listening on: {}", listener.local_addr()?);

    // Start serving with graceful shutdown
    // Errors are handled by the global error handler middleware
    serve(listener, app)
        .with_graceful_shutdown(server::shutdown_signal())
        .await?;

    Ok(())
}
```

### 2.2. Core Modules

#### Server Configuration (`src/core/server.rs`)

```rust
// Create and configure the application router with all middleware layers
pub fn create_app(state: AppState) -> Router {
    let env = EnvironmentVariables::instance();
    
    Router::new()
        .merge(test_routes())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(response_wrapper))
                .layer(HandleErrorLayer::new(handle_global_error))
                .layer(TimeoutLayer::new(Duration::from_secs(env.default_timeout_seconds)))
                .layer(DefaultBodyLimit::max(env.max_request_body_size))
        )
        .with_state(state)
}
```

#### Logging Configuration (`src/core/logging.rs`)

```rust
// Initialize the tracing subscriber with default configuration
pub fn init_tracing() {
    let env_filter: EnvFilter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "my_axum_project=info,tower_http=debug,axum=trace".parse().unwrap());
    
    fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::FULL)
        .init();
}
```

---

## 3. Configuration System

### 3.1. Layered Environment Configuration

The template uses a sophisticated configuration system with priority:

```
System Variables > .env.local/.env.production > .env (base)
```

**Base configuration** (`.env` - included in repository):
```bash
# Application Environment
ENVIRONMENT=development

# Server Configuration
HOST=127.0.0.1
PORT=3000
PROTOCOL=http

# Request Configuration
MAX_REQUEST_BODY_SIZE=2097152
DEFAULT_TIMEOUT_SECONDS=30

# Database Configuration (development defaults)
DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=postgres
```

**Local overrides** (`.env.local` - gitignored):
```bash
# Copy .env.local.example to .env.local for local overrides
PORT=8080
DEFAULT_TIMEOUT_SECONDS=60
DB_PASSWORD=my_secure_password
```

### 3.2. Environment Variables Singleton

```rust
// Loads environment variables with priority: .env < .env.local < .env.production
// Always loads .env as base configuration, then overrides with local/production files
impl EnvironmentVariables {
    fn load() -> Result<Self> {
        let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| DEFAULT_ENVIRONMENT.to_string());
        
        // Load base configuration from .env
        if let Err(e) = dotenv::from_path(".env") {
            warn!("Could not load .env file: {}", e);
        }
        
        // Load environment-specific overrides
        match environment.as_str() {
            "production" => {
                if let Err(e) = dotenv::from_path(".env.production") {
                    warn!("Could not load .env.production file: {}", e);
                }
            }
            _ => {
                // In development, load .env.local for local overrides
                if let Err(e) = dotenv::from_path(".env.local") {
                    tracing::debug!("No .env.local file found: {}", e);
                }
            }
        }

        // Build configuration with defaults
        Ok(Self { /* ... */ })
    }

    // Returns a reference to the lazily-initialized environment configuration
    pub fn instance() -> &'static Self {
        static INSTANCE: Lazy<Result<EnvironmentVariables, anyhow::Error>> = Lazy::new(|| {
            let config: EnvironmentVariables = EnvironmentVariables::load()?;
            
            if cfg!(debug_assertions) {
                tracing::debug!("Loaded environment configuration: {:#?}", config);
            }
            
            Ok(config)
        });

        // Panics if loading fails
        INSTANCE.as_ref().expect("Failed to load environment configuration")
    }
}
```

---

## 4. Global Error Handling

### 4.1. Error Handler (`src/utils/error_handler/error_handler.rs`)

The global error handler provides comprehensive error handling for common HTTP error scenarios:

```rust
// Global error handling logic for layers (e.g. timeouts, large payloads, not found).

use axum::{
    BoxError,
    http::StatusCode,
    response::IntoResponse,
    extract::rejection::MatchedPathRejection,
};
use std::error::Error;
use tower::timeout::error::Elapsed;
use http_body_util::LengthLimitError;

// This is the main function that maps errors to HTTP responses
pub async fn handle_global_error(err: BoxError) -> impl IntoResponse {
    // 413 if the body was too large
    if find_cause::<LengthLimitError>(&*err).is_some() {
        return StatusCode::PAYLOAD_TOO_LARGE;
    }

    // 408 if the request took too long
    if err.is::<Elapsed>() {
        return StatusCode::REQUEST_TIMEOUT;
    }

    // 404 for not found routes/resources
    if find_cause::<MatchedPathRejection>(&*err).is_some() {
        return StatusCode::NOT_FOUND;
    }

    // Check for common not found errors that should be 404
    let error_msg = err.to_string().to_lowercase();
    if error_msg.contains("not found") 
        || error_msg.contains("no route found")
        || error_msg.contains("route not found")
        || error_msg.contains("path not found")
        || error_msg.contains("resource not found") {
        return StatusCode::NOT_FOUND;
    }

    // Otherwise, 500
    StatusCode::INTERNAL_SERVER_ERROR
}

// A small helper function to find a specific cause in a chain of errors
pub fn find_cause<T: Error + 'static>(err: &dyn Error) -> Option<&T> {
    let mut source: Option<&dyn Error> = err.source();
    
    while let Some(s) = source {
        if let Some(typed) = s.downcast_ref::<T>() {
            return Some(typed);
        }
        source = s.source();
    }

    None
}
```

### 4.2. Error Types Handled

The global error handler provides explicit handling for common HTTP error scenarios:

#### **404 Not Found**
- **`MatchedPathRejection`** → Route not found in the router
- **Not found patterns** → Detected by error message patterns (not found, route not found, etc.)

#### **408 Request Timeout**
- **`Elapsed`** → Request exceeded the configured timeout limit

#### **413 Payload Too Large**
- **`LengthLimitError`** → Request body exceeds the configured size limit

#### **500 Internal Server Error**
- **Any other error** → Fallback for unhandled errors

### 4.3. Testing Error Handling

The API includes test endpoints to validate each error type:

```bash
# Test 404 Not Found
curl http://localhost:3000/test/not-found

# Test 408 Request Timeout (will take ~4 seconds)
curl http://localhost:3000/test/timeout

# Test 413 Payload Too Large (requires payload > 2MB)
python3 -c "import json; print(json.dumps({'data': 'x' * 3000000}))" | \
curl -X POST http://localhost:3000/test/body-size \
  -H "Content-Type: application/json" \
  -d @-

# Test 500 Internal Server Error
curl http://localhost:3000/test/error
```

### 4.4. Consistent Error Responses

All errors are automatically wrapped in the standard response format:

```json
{
  "status": "NOT_FOUND",
  "code": 404,
  "data": null,
  "messages": [],
  "date": "2025-06-02T20:59:03.015201+00:00"
}
```

---

## 5. Unified Response System

### 5.1. HandlerResponse Structure

```rust
// A convenience struct that can be returned from handlers
#[derive(Debug, Clone)]
pub struct HandlerResponse {
    pub status_code: StatusCode,
    pub data: serde_json::Value,
    pub messages: Vec<String>,
}

impl HandlerResponse {
    // Initialize with a status code, defaulting data = null, messages = []
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            data: serde_json::Value::Null,
            messages: Vec::new(),
        }
    }

    // Add a JSON data object
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    // Add a message string
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.messages.push(message.into());
        self
    }
}
```

### 5.2. Response Wrapper Middleware

The `response_wrapper` middleware automatically formats all responses:

```rust
// The main middleware that wraps every response in ResponseFormat
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    // Run the next service (handler or next layer)
    let response: Response<Body> = next.run(req).await;

    // Extract the HandlerResponse fields (messages, data)
    let (messages, data) = extract_response_components(&response);

    // Deconstruct the response into parts
    let (parts, _) = response.into_parts();

    // Build the final top-level JSON
    let default_status: String = create_default_status_message(&parts);
    let formatted_status: String = default_status.to_uppercase().replace(' ', "_");

    let wrapped: ResponseFormat = ResponseFormat {
        status: formatted_status,
        code: parts.status.as_u16(),
        data,
        messages,
        date: Utc::now().to_rfc3339(),
    };

    // Log the final JSON structure
    log_formatted_response(&wrapped);

    // Convert parts + wrapped data into a final Response
    Ok(build_final_response(parts, &wrapped))
}
```

---

## 6. Adding New Endpoints

### 6.1. Create a New API Module

**Step 1**: Create the directory structure:
```bash
mkdir -p src/api/users
```

**Step 2**: Create `src/api/users/mod.rs`:
```rust
/*
    The users API module. Re-exports the routes and handler.
*/

pub mod handler;
pub mod routes;
```

**Step 3**: Create `src/api/users/handler.rs`:
```rust
// Demonstrates user management endpoints

use serde_json::json;
use axum::{http::StatusCode, extract::{State, Path}, body::Bytes};
use std::backtrace::Backtrace;

use crate::config::state::AppState;
use crate::utils::response_handler::HandlerResponse;
use tracing::instrument;

#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn get_users(
    State(_state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "users": [] }))
        .message("Users retrieved successfully")
}

#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn get_user(
    State(_state): State<AppState>,
    Path(user_id): Path<u32>,
    _body: Bytes,
) -> HandlerResponse {
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "user_id": user_id, "name": "John Doe" }))
        .message("User retrieved successfully")
}
```

**Step 4**: Create `src/api/users/routes.rs`:
```rust
// Defines the user routes

use axum::{routing::get, Router};
use crate::api::users::handler::{get_users, get_user};
use crate::config::state::AppState;

// Build a Router with user-related routes
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_users))
        .route("/users/:user_id", get(get_user))
}
```

**Step 5**: Update `src/api/mod.rs`:
```rust
/*
    API endpoints and routes module.
    Re-export all API-related modules here.
*/

pub mod hello;
pub mod users;  // Add this line
```

**Step 6**: Update `src/core/server.rs`:
```rust
use crate::api::users::routes::user_routes;  // Add this import

// In create_app function
Router::new()
    .merge(hello_routes())
    .merge(user_routes())  // Add this line
    .layer(/* ... */)
```

### 6.2. Testing New Endpoints

```bash
# Get all users
curl http://127.0.0.1:3000/users

# Get specific user
curl http://127.0.0.1:3000/users/123
```

---

## 7. Development Workflow

### 7.1. Hot Reload Setup

Install required tools:
```bash
cargo install cargo-watch systemfd
```

Run with hot reload:
```bash
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

### 7.2. Environment Configuration

**Development**:
- Uses `.env` as base configuration
- Create `.env.local` for personal overrides

**Production**:
- Uses system environment variables
- Create `.env.production` for production-specific settings

### 7.3. Logging Configuration

Customize logging levels via environment variables:
```bash
# In .env.local
RUST_LOG=debug,my_axum_project=trace,tower_http=info
```

---

## 8. Best Practices

### 8.1. Error Handling

- Use `HandlerResponse` for consistent API responses
- Let the global error handler manage layer errors (timeouts, body limits)
- Use `?` operator in main.rs for critical bootstrap errors

### 8.2. Configuration Management

- Keep sensitive data in `.env.local` or `.env.production` (gitignored)
- Use meaningful defaults in the base `.env` file
- Access configuration through the singleton pattern

### 8.3. Code Organization

- Keep handlers focused on business logic
- Use the response wrapper for consistent formatting
- Organize features by API domain (`api/users/`, `api/posts/`, etc.)
- Put shared utilities in `utils/`
- Keep core functionality in `core/`

### 8.4. Testing

- Test individual handlers independently
- Use the configured middleware stack for integration tests
- Mock external dependencies through dependency injection

---

## 9. Dependencies

### Core Dependencies
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.8.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower = { version = "0.5.2", features = ["util", "timeout"] }
http-body-util = "0.1.3"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = "0.4"
listenfd = "1.0.2"
dotenv = "0.15"
anyhow = "1.0"
once_cell = "1.18.0"
```

---

## 10. Deployment

### 10.1. Production Configuration

Set `ENVIRONMENT=production` to:
- Skip loading `.env` files
- Use only system environment variables
- Enable production optimizations

### 10.2. Docker Example

```dockerfile
FROM rust:1.85 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my-axum-project /usr/local/bin/my-axum-project
ENV ENVIRONMENT=production
EXPOSE 3000
CMD ["my-axum-project"]
```

---

## Conclusion

This template provides a solid foundation for building production-ready APIs with Rust and Axum. The modular architecture, comprehensive error handling, and flexible configuration system make it easy to extend and maintain.

Key features:
- ✅ **Zero-setup development** experience
- ✅ **Production-ready** configuration management
- ✅ **Consistent API responses** with automatic formatting
- ✅ **Comprehensive error handling** with proper HTTP status codes
- ✅ **Hot reload** for efficient development
- ✅ **Graceful shutdown** for reliable deployments
- ✅ **Structured logging** for observability

Happy coding! 🦀 