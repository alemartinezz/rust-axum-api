# 🛠️ Utils Module

This module provides essential utility functions and cross-cutting concerns that support the entire application infrastructure. It includes unified response handling, comprehensive error management, and common utility functions.

## 📂 Module Structure

```
src/utils/
├── mod.rs                        # Module exports and utilities aggregation
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

## 📤 Response Handler System (`response_handler/`)

### **Unified Response Format**
All API responses follow a consistent JSON structure for predictable client integration:

```json
{
  "status": "OK",                    // HTTP status text (uppercase with underscores)
  "code": 200,                       // HTTP status code
  "data": { /* payload */ },         // Response data (any JSON type)
  "messages": ["Success message"],   // Informational messages array
  "date": "2024-01-01T00:00:00Z"    // ISO timestamp (UTC)
}
```

### **ResponseFormat Structure**
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

### **HandlerResponse Builder Pattern**
Provides a convenient API for constructing responses in handlers:

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

### **Usage Examples**
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

### **Middleware Integration**
The response wrapper middleware automatically processes all responses:

```rust
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let response = next.run(req).await;
    
    // Extract HandlerResponse from extensions
    let (messages, data) = extract_response_components(&response);
    
    // Wrap in standard format
    let wrapped = ResponseFormat {
        status: formatted_status,
        code: parts.status.as_u16(),
        data,
        messages,
        date: Utc::now().to_rfc3339(),
    };
    
    // Log and return formatted response
    log_formatted_response(&wrapped);
    Ok(build_final_response(parts, &wrapped))
}
```

### **Response Processing Features**
- **Automatic Wrapping**: All responses automatically formatted
- **Content-Type Setting**: Automatic `application/json` headers
- **Structured Logging**: Pretty-printed JSON logging with indentation
- **Extension-Based**: Uses HTTP response extensions for data passing
- **Status Normalization**: Converts HTTP status to consistent string format

## 🚨 Error Handler System (`error_handler/`)

### **Global Error Mapping**
Provides intelligent error type detection and appropriate HTTP status code mapping:

```rust
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

    // Otherwise, 500
    StatusCode::INTERNAL_SERVER_ERROR
}
```

### **Error Type Mappings**

| Error Type | HTTP Status | Description |
|------------|-------------|-------------|
| `LengthLimitError` | 413 PAYLOAD_TOO_LARGE | Request body exceeds size limit |
| `Elapsed` | 408 REQUEST_TIMEOUT | Request processing timeout |
| `MatchedPathRejection` | 404 NOT_FOUND | Route not found |
| `Default` | 500 INTERNAL_SERVER_ERROR | Unexpected server errors |

### **Error Chain Traversal**
Sophisticated error cause detection using recursive source chain analysis:

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

### **Integration with Tower**
Works seamlessly with Tower's middleware error handling:
- **HandleErrorLayer**: Catches errors from inner layers
- **BoxError**: Handles any error type through trait objects
- **Error Propagation**: Maintains error context through middleware stack

## 🧰 General Utilities (`utils/`)

### **JSON Formatting Utilities**
```rust
// Pretty-print JSON with custom indentation
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

### **Common Utilities**
- **String manipulation**: Case conversion, validation helpers
- **Date/time helpers**: UTC timestamp generation, formatting
- **Validation functions**: Input sanitization, format checking
- **Conversion utilities**: Type conversions, data transformations

## 🏗️ Architecture Integration

### **Middleware Stack Position**
The utils components integrate at specific middleware layers:

```
Request Flow:
┌─────────────────┐
│   Incoming      │
│   Request       │
└─────────────────┘
         ↓
┌─────────────────┐
│  Body Limit     │  ← error_handler (LengthLimitError)
└─────────────────┘
         ↓
┌─────────────────┐
│   Timeout       │  ← error_handler (Elapsed)
└─────────────────┘
         ↓
┌─────────────────┐
│ Global Error    │  ← error_handler (all errors)
└─────────────────┘
         ↓
┌─────────────────┐
│ Response Wrap   │  ← response_handler (standardization)
└─────────────────┘
         ↓
┌─────────────────┐
│   HTTP          │
│   Response      │
└─────────────────┘
```

### **Cross-Module Dependencies**
```rust
// API handlers use response utilities
use crate::utils::response_handler::HandlerResponse;

// Core server uses error handling
use crate::utils::error_handler::handle_global_error;

// Database operations use formatting utilities
use crate::utils::utils::to_two_space_indented_json;
```

## 📊 Response Examples

### **Successful API Response**
```json
{
  "status": "OK",
  "code": 200,
  "data": {
    "tenants": [
      {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "tenant_name": "acme_corp",
        "schema_name": "tenant_123e4567",
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "count": 1
  },
  "messages": ["Tenants retrieved successfully"],
  "date": "2024-01-01T12:34:56Z"
}
```

### **Error Response**
```json
{
  "status": "BAD_REQUEST",
  "code": 400,
  "data": {
    "error": "tenant_name_invalid",
    "details": "Tenant name can only contain alphanumeric characters"
  },
  "messages": ["Invalid tenant name format"],
  "date": "2024-01-01T12:34:56Z"
}
```

### **Timeout Error (Automatic)**
```json
{
  "status": "REQUEST_TIMEOUT",
  "code": 408,
  "data": null,
  "messages": [],
  "date": "2024-01-01T12:34:56Z"
}
```

## 🔧 Configuration & Customization

### **Response Format Customization**
The response format can be extended for specific needs:
```rust
// Custom response fields
#[derive(Serialize)]
pub struct ExtendedResponseFormat {
    #[serde(flatten)]
    pub base: ResponseFormat,
    pub request_id: String,
    pub processing_time_ms: u64,
}
```

### **Error Handler Extension**
Adding custom error types:
```rust
// Custom error mapping
if find_cause::<CustomBusinessError>(&*err).is_some() {
    return StatusCode::UNPROCESSABLE_ENTITY;
}
```

### **Utility Function Extension**
```rust
// Add new utilities in utils_impl.rs
pub fn validate_tenant_name(name: &str) -> Result<(), ValidationError> {
    // Custom validation logic
}

pub fn generate_schema_name(tenant_id: &Uuid) -> String {
    format!("tenant_{}", tenant_id.simple())
}
```

## 🚀 Performance Considerations

### **Response Processing Efficiency**
- **Extension-Based**: Avoids response body parsing for data extraction
- **Single Serialization**: JSON serialized only once in middleware
- **Memory Efficient**: Minimal allocations in response path
- **Lazy Formatting**: Pretty-printing only for logging

### **Error Handling Performance**
- **Early Detection**: Error types detected without full traversal when possible
- **Cache-Friendly**: Error type checks use efficient downcast operations
- **Minimal Overhead**: Error handling adds minimal latency to success path

### **Logging Optimization**
- **Conditional Formatting**: Pretty-printing only when logging is enabled
- **Structured Output**: Machine-readable logs for production environments
- **Async Logging**: Non-blocking log output using tracing infrastructure

## 🧪 Testing Support

### **Response Testing Utilities**
```rust
#[cfg(test)]
pub fn assert_response_format(response: &Response<Body>) {
    // Validate response follows standard format
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
    // Additional format validations
}
```

### **Error Simulation**
```rust
#[cfg(test)]
pub fn simulate_timeout_error() -> BoxError {
    Box::new(Elapsed::new())
}

#[cfg(test)]
pub fn simulate_body_limit_error() -> BoxError {
    Box::new(LengthLimitError)
}
```

## 📈 Monitoring & Observability

### **Response Logging**
All responses are logged with structured formatting:
```
INFO Final response:
{
  "status": "CREATED",
  "code": 201,
  "data": {
    "tenant_id": "123",
    "name": "acme_corp"
  },
  "messages": ["Tenant created successfully"],
  "date": "2024-01-01T12:34:56Z"
}
```

### **Error Tracking**
- **Error Classification**: Automatic categorization by error type
- **Context Preservation**: Original error context maintained through chain
- **Metrics Integration**: Error counts and types tracked for monitoring
- **Debug Information**: Full error details available in development

## 🏆 Best Practices

### **Response Construction**
- **Consistent Structure**: Always use HandlerResponse builder
- **Meaningful Messages**: Provide clear, actionable error messages
- **Appropriate Status Codes**: Use correct HTTP status for each scenario
- **Rich Data**: Include relevant context in response data

### **Error Handling**
- **Graceful Degradation**: Provide meaningful errors for all failure modes
- **Security Awareness**: Don't leak sensitive information in error responses
- **User-Friendly**: Error messages should be understandable by API consumers
- **Logging Integration**: Ensure errors are properly logged for debugging

### **Utility Usage**
- **Reusability**: Design utilities for multiple use cases
- **Performance**: Consider performance impact of utility functions
- **Testing**: Provide test utilities for common scenarios
- **Documentation**: Clear documentation for utility function behavior

This utils module provides the essential infrastructure for consistent, reliable, and maintainable API responses and error handling across the entire application. 