# Utils Module

Essential utility functions and cross-cutting concerns supporting the entire application infrastructure.

## Structure

```
src/utils/
├── mod.rs                        # Module exports
├── response_handler/             # Unified response system
│   ├── mod.rs                   # Response handler exports
│   └── response_handler.rs      # Response formatting and middleware
├── error_handler/               # Global error management
│   ├── mod.rs                   # Error handler exports
│   └── error_handler.rs         # Error type mapping and handling
└── utils/                       # General utility functions
    ├── mod.rs                   # Utils exports
    └── utils_impl.rs            # Common utility implementations
```

## Response Handler System

### Unified Response Format
All API responses follow a consistent JSON structure:

```json
{
  "status": "OK",                    // HTTP status text
  "code": 200,                       // HTTP status code
  "data": { /* payload */ },         // Response data (any JSON type)
  "messages": ["Success message"],   // Informational messages array
  "date": "2024-01-01T00:00:00Z"    // ISO timestamp (UTC)
}
```

### ResponseFormat Structure
```rust
#[derive(Serialize, Deserialize)]
pub struct ResponseFormat {
    pub status: String,              // "OK", "NOT_FOUND", "INTERNAL_SERVER_ERROR"
    pub code: u16,                   // 200, 404, 500, etc.
    pub data: serde_json::Value,     // Flexible JSON payload
    pub messages: Vec<String>,       // Multiple informational messages
    pub date: String,                // RFC3339 formatted timestamp
}
```

### HandlerResponse Builder
Convenient API for constructing responses in handlers:

```rust
#[derive(Debug, Clone)]
pub struct HandlerResponse {
    pub status_code: StatusCode,
    pub data: serde_json::Value,
    pub messages: Vec<String>,
}

impl HandlerResponse {
    pub fn new(status_code: StatusCode) -> Self
    pub fn data(self, data: serde_json::Value) -> Self
    pub fn message(self, message: impl Into<String>) -> Self
}
```

### Usage Examples
```rust
// Success response with data
HandlerResponse::new(StatusCode::OK)
    .data(json!({"tenant_id": "123", "name": "acme_corp"}))
    .message("Tenant retrieved successfully")

// Error response
HandlerResponse::new(StatusCode::BAD_REQUEST)
    .data(json!({"error": "invalid_tenant_name"}))
    .message("Tenant name cannot be empty")

// Multiple messages
HandlerResponse::new(StatusCode::CREATED)
    .data(json!(tenant_data))
    .message("Tenant created successfully")
    .message("Welcome email sent")
```

### Middleware Integration
The response wrapper middleware automatically processes all responses:

```rust
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let response = next.run(req).await;
    
    // Extract response components and wrap in standard format
    let wrapped = ResponseFormat {
        status: formatted_status,
        code: parts.status.as_u16(),
        data: extracted_data,
        messages: extracted_messages,
        date: Utc::now().to_rfc3339(),
    };
    
    log_formatted_response(&wrapped);
    Ok(build_final_response(parts, &wrapped))
}
```

### Features
- **Automatic Wrapping**: All responses automatically formatted
- **Content-Type Setting**: Automatic `application/json` headers
- **Structured Logging**: Pretty-printed JSON logging
- **Extension-Based**: Uses HTTP response extensions for data passing
- **Status Normalization**: Converts HTTP status to consistent string format

## Error Handler System

### Global Error Mapping
Intelligent error type detection and HTTP status code mapping:

```rust
pub async fn handle_global_error(err: BoxError) -> impl IntoResponse {
    // Request body too large
    if find_cause::<LengthLimitError>(&*err).is_some() {
        return StatusCode::PAYLOAD_TOO_LARGE;
    }

    // Request timeout
    if err.is::<Elapsed>() {
        return StatusCode::REQUEST_TIMEOUT;
    }

    // Route not found
    if find_cause::<MatchedPathRejection>(&*err).is_some() {
        return StatusCode::NOT_FOUND;
    }

    // Default to internal server error
    StatusCode::INTERNAL_SERVER_ERROR
}
```

### Error Type Mappings

| Error Type | HTTP Status | Description |
|------------|-------------|-------------|
| `LengthLimitError` | 413 PAYLOAD_TOO_LARGE | Request body exceeds size limit |
| `Elapsed` | 408 REQUEST_TIMEOUT | Request processing timeout |
| `MatchedPathRejection` | 404 NOT_FOUND | Route not found |
| `Default` | 500 INTERNAL_SERVER_ERROR | Unexpected server errors |

### Error Chain Traversal
Sophisticated error cause detection:

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

### Tower Integration
- **HandleErrorLayer**: Catches errors from inner layers
- **BoxError**: Handles any error type through trait objects
- **Error Propagation**: Maintains error context through middleware stack

## General Utilities

### JSON Formatting
```rust
// Pretty-print JSON with 2-space indentation
pub fn to_two_space_indented_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let pretty = serde_json::to_string_pretty(value)?;
    // Convert from 4-space to 2-space indentation
    Ok(pretty.lines()
        .map(|line| {
            let leading_spaces = line.len() - line.trim_start().len();
            format!("{}{}", " ".repeat(leading_spaces / 2), line.trim_start())
        })
        .collect::<Vec<_>>()
        .join("\n"))
}
```

### Common Functions
- **String manipulation**: Case conversion, validation helpers
- **Date/time helpers**: UTC timestamp generation, formatting
- **Validation utilities**: Input sanitization and checking
- **Response helpers**: Status code and header utilities

## Features

- **Unified Responses**: Consistent JSON format across all endpoints
- **Error Mapping**: Intelligent HTTP status code assignment
- **Middleware Integration**: Seamless Tower middleware compatibility
- **Structured Logging**: Pretty-printed response logging
- **Type Safety**: Strong typing for response building
- **Extension System**: HTTP response extensions for data passing
- **Performance**: Efficient JSON processing and formatting

## Usage in Application

### Handler Implementation
```rust
async fn create_tenant(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTenantRequest>
) -> HandlerResponse {
    match tenant_service.create(&payload.tenant_name).await {
        Ok(tenant) => {
            HandlerResponse::new(StatusCode::CREATED)
                .data(json!(tenant))
                .message("Tenant created successfully")
        }
        Err(e) => {
            HandlerResponse::new(StatusCode::BAD_REQUEST)
                .data(json!({"error": e.to_string()}))
                .message("Failed to create tenant")
        }
    }
}
```

### Error Handling
```rust
// Automatic error mapping in middleware
app.layer(HandleErrorLayer::new(handle_global_error))

// Custom error types automatically mapped to appropriate HTTP status
```

This module provides the foundation for consistent API responses and robust error handling throughout the application. 