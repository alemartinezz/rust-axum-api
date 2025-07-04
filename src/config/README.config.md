# Configuration Module

Manages application configuration, environment variables, and global state using a singleton pattern.

## Structure

```
src/config/
├── mod.rs              # Module exports
├── environment.rs      # Environment variable management
└── state.rs           # Application state singleton
```

## Environment Management

### Configuration Hierarchy
```
1. System Environment Variables (highest priority)
2. .env.production (production only)
3. .env.local (development overrides, gitignored)
4. .env (base configuration)
```

### Environment Variables
```rust
pub struct EnvironmentVariables {
    // Server
    pub environment: Cow<'static, str>,     // development|production
    pub host: Cow<'static, str>,            // Server host
    pub port: u16,                          // Server port
    pub protocol: Cow<'static, str>,        // http|https
    
    // Request limits
    pub max_request_body_size: usize,       // Bytes
    pub default_timeout_seconds: u64,       // Seconds
    
    // Database
    pub db_host: Cow<'static, str>,         // PostgreSQL host
    pub db_port: u16,                       // PostgreSQL port
    pub db_name: Cow<'static, str>,         // Database name
    pub db_user: Cow<'static, str>,         // Database user
    pub db_password: Cow<'static, str>,     // Database password
}
```

## Application State

### Singleton Pattern
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

### Usage
```rust
// Access singleton
let state = AppState::instance();
let db = &state.database;
let config = &state.environment;

// In Axum handlers (auto-injected)
async fn handler(State(state): State<AppState>) -> Response {
    // state is available here
}
```

## Configuration Files

### Base (.env)
```env
ENVIRONMENT=development
HOST=localhost
PORT=3000
DB_HOST=localhost
DB_PORT=5432
DB_NAME=rust_axum_api
DB_USER=postgres
DB_PASSWORD=postgres
```

### Development (.env.local)
```env
# Local overrides
DB_PASSWORD=dev_password
PORT=3001
```

### Production (.env.production)
```env
# Production settings
PROTOCOL=https
DB_HOST=prod-db.company.com
```

## Features

- **Validation**: Type checking and format validation
- **Error Reporting**: Comprehensive missing variable reports
- **Thread Safety**: Arc-based sharing across async tasks
- **Memory Efficient**: Cow<'static, str> for zero-copy strings
- **Lazy Loading**: Initialized only when first accessed

## Usage Example

```rust
// Initialize (done automatically by AppState)
let config = EnvironmentVariables::load()?;

// Access through singleton
let state = AppState::instance();
let timeout = state.environment.default_timeout_seconds;
let db_host = &state.environment.db_host;
``` 