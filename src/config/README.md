# ⚙️ Configuration Module

This module manages all application configuration, environment variables, and global state using sophisticated patterns for scalable, maintainable configuration management.

## 📂 Module Structure

```
src/config/
├── mod.rs              # Module exports
├── environment.rs      # Environment variable management
└── state.rs           # Application state singleton
```

## 🌍 Environment Management (`environment.rs`)

### **Layered Configuration Loading**
The system implements a sophisticated environment loading hierarchy:

```
Priority (highest to lowest):
1. System Environment Variables
2. .env.production (if ENVIRONMENT=production)
3. .env.local (if ENVIRONMENT=development)
4. .env (base configuration)
```

### **Environment Variables Structure**
```rust
pub struct EnvironmentVariables {
    // Server Configuration
    pub environment: Cow<'static, str>,          // development|staging|production
    pub host: Cow<'static, str>,                 // Server bind address
    pub port: u16,                               // Server port
    pub protocol: Cow<'static, str>,             // http|https
    
    // Request Handling
    pub max_request_body_size: usize,            // Bytes limit
    pub default_timeout_seconds: u64,            // Request timeout
    
    // Database Configuration
    pub db_host: Cow<'static, str>,              // PostgreSQL host
    pub db_port: u16,                            // PostgreSQL port
    pub db_name: Cow<'static, str>,              // Database name
    pub db_user: Cow<'static, str>,              // Database user
    pub db_password: Cow<'static, str>,          // Database password
}
```

### **Environment-Specific Loading**
```rust
// Development: .env → .env.local (local overrides)
// Production:  .env → .env.production (prod overrides)
// Other:       .env only (staging, testing, etc.)
```

### **Validation Features**
- **Missing Variable Detection**: Comprehensive error reporting
- **Format Validation**: Type-specific parsing with descriptive errors
- **Value Range Checking**: Port numbers, protocol values, etc.
- **Batch Error Reporting**: All issues reported at once

### **Error Handling Examples**
```
Missing required environment variables:
  - DB_PASSWORD
  - HOST

Incorrect format environment variables:
  - PORT (current: "abc", should be: numeric value between 1-65535)
  - PROTOCOL (current: "ftp", should be: "http" or "https")
```

## 🏛️ Application State (`state.rs`)

### **Singleton Pattern Implementation**
```rust
#[derive(Debug, Clone)]
pub struct AppState {
    pub environment: Arc<EnvironmentVariables>,
    pub database: DatabaseService,
}

// Thread-safe singleton using once_cell
static INSTANCE: Lazy<AppState> = Lazy::new(|| {
    AppState::new().expect("Failed to initialize AppState")
});
```

### **Key Features**
- **Thread-Safe**: `Arc` sharing for concurrent access
- **Lazy Initialization**: Created only when first accessed
- **Clone-Friendly**: Cheap cloning via `Arc` references
- **Error Propagation**: Clear initialization failure handling

### **Usage Pattern**
```rust
// In handlers and middleware
let state = AppState::instance();
let db_service = &state.database;
let env_vars = &state.environment;

// In Axum routes (automatically injected)
pub async fn handler(State(state): State<AppState>) -> Response {
    // state is already available
}
```

## 🚀 Configuration Files

### **Base Configuration (`.env`)**
```env
# Environment
ENVIRONMENT=development

# Server
HOST=localhost
PORT=3000
PROTOCOL=http
MAX_REQUEST_BODY_SIZE=1048576
DEFAULT_TIMEOUT_SECONDS=30

# Database
DB_HOST=localhost
DB_PORT=5432
DB_NAME=rust_axum_api
DB_USER=postgres
DB_PASSWORD=postgres
```

### **Local Development (`.env.local`)**
```env
# Override for local development
DB_PASSWORD=dev_password
PORT=3001
```

### **Production Configuration (`.env.production`)**
```env
# Production overrides
PROTOCOL=https
DB_HOST=prod-db.company.com
MAX_REQUEST_BODY_SIZE=2097152
DEFAULT_TIMEOUT_SECONDS=60
```

## 🏗️ Architecture Patterns

### **Dependency Injection**
The AppState serves as a dependency injection container:
```rust
// Database service gets environment configuration
pub fn new(config: Arc<EnvironmentVariables>) -> Self {
    Self {
        environment: environment_arc.clone(),
        database: DatabaseService::new(environment_arc),
    }
}
```

### **Configuration Sharing**
```rust
// Efficient sharing via Arc
pub environment: Arc<EnvironmentVariables>,

// Multiple components can share the same config
let db_config = config.clone();  // Just increments reference count
let server_config = config.clone();
```

### **Lifecycle Management**
```rust
// Initialization
AppState::init_master_schema().await?;

// Runtime access
let state = AppState::instance();

// Shutdown
AppState::shutdown().await;
```

## 🔒 Security & Validation

### **Environment Validation**
- **Required Variables**: All variables must be present
- **Type Safety**: Compile-time and runtime type checking
- **Format Validation**: Strict format requirements
- **Range Checking**: Numerical bounds validation

### **Memory-Efficient Strings**
```rust
// Uses Cow<'static, str> for efficient string handling
pub host: Cow<'static, str>,  // Zero-copy for static strings

// Automatic conversion from environment
let host = check_var("HOST", &mut missing_vars)
    .map(|s: String| Cow::<'static, str>::Owned(s));
```

### **Production Security**
- **No Default Secrets**: All sensitive values must be explicitly set
- **Environment Separation**: Different configs for different environments
- **Validation Logging**: Configuration errors are logged securely

## 📊 Configuration Examples

### **Development Setup**
```rust
// Loads: .env → .env.local
EnvironmentVariables {
    environment: "development",
    host: "localhost",
    port: 3001,  // Overridden in .env.local
    protocol: "http",
    db_password: "dev_password",  // Overridden
    // ... other fields
}
```

### **Production Setup**
```rust
// Loads: .env → .env.production
EnvironmentVariables {
    environment: "production",
    host: "0.0.0.0",
    port: 3000,
    protocol: "https",  // Overridden for production
    max_request_body_size: 2097152,  // Larger in prod
    // ... other fields
}
```

## 🚀 Usage in Application

### **Server Configuration**
```rust
pub async fn setup_listener() -> Result<TcpListener> {
    let env = &AppState::instance().environment;
    let addr = format!("{}:{}", env.host, env.port);
    TcpListener::bind(&addr).await
}
```

### **Database Configuration**
```rust
async fn create_connect_options(&self, schema: Option<&str>) -> Result<PgConnectOptions> {
    let options = PgConnectOptions::new()
        .host(&self.config.db_host)
        .port(self.config.db_port)
        .username(&self.config.db_user)
        .password(&self.config.db_password)
        .database(&self.config.db_name);
}
```

### **Middleware Configuration**
```rust
.layer(TimeoutLayer::new(Duration::from_secs(env.default_timeout_seconds)))
.layer(DefaultBodyLimit::max(env.max_request_body_size))
```

## 🔄 Hot Reload Support

### **Development Workflow**
1. **File Watching**: `cargo-watch` monitors file changes
2. **Graceful Restart**: `listenfd` maintains connections
3. **State Reinitialization**: New config loaded on restart
4. **Zero Downtime**: Seamless configuration updates

### **Configuration Changes**
```bash
# Change .env.local
echo "PORT=3002" >> .env.local

# Auto-restart picks up changes
# New AppState instance with updated port
```

## 📈 Performance Optimizations

### **Lazy Loading**
- **Singleton Creation**: Only created when first accessed
- **Arc Sharing**: Zero-copy sharing between threads
- **Static Lifetime**: No runtime allocation overhead

### **Memory Efficiency**
```rust
// Cow<'static, str> optimizations
pub environment: Cow<'static, str>,  // "development" → static reference
pub host: Cow<'static, str>,         // "localhost" → static reference
```

### **Validation Caching**
- **Single Parse**: Environment parsed once at startup
- **Type Conversion**: All parsing done during initialization
- **Error Batching**: Multiple errors collected in single pass

## 🧪 Testing Support

### **Test Configuration**
```rust
#[cfg(test)]
impl EnvironmentVariables {
    pub fn test_config() -> Self {
        // Minimal config for testing
    }
}
```

### **Mocking Support**
```rust
// AppState can be injected for testing
pub fn with_test_state<F>(f: F) where F: FnOnce(&AppState) {
    let test_state = AppState::test_instance();
    f(&test_state);
}
```

## 🏆 Best Practices

### **Configuration Management**
- **Environment Separation**: Different configs per environment
- **Validation First**: Fail fast on invalid configuration
- **Comprehensive Errors**: Clear error messages for debugging
- **Security Defaults**: Safe defaults, explicit overrides

### **State Management**
- **Singleton Access**: Consistent state across application
- **Arc Sharing**: Efficient memory usage
- **Lifecycle Hooks**: Proper initialization and cleanup
- **Thread Safety**: Safe concurrent access patterns

This configuration module provides enterprise-grade configuration management with sophisticated validation, efficient memory usage, and robust error handling for production applications. 