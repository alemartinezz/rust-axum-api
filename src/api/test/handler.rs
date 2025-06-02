// Start of file: /src/api/test/handler.rs

// Test endpoints for validating different middleware and functionalities

use serde_json::json;
use axum::{http::StatusCode, extract::State, body::Bytes};
use std::backtrace::Backtrace;

use crate::config::{state::AppState, environment::EnvironmentVariables};
use crate::utils::response_handler::HandlerResponse;
use tracing::{instrument, info};

// Simple hello endpoint (original functionality)
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn hello_handler(
    State(_state): State<AppState>,
    _body: Bytes,        // This forces Axum to read the body, also triggers body-size limits
) -> HandlerResponse {
    info!("Hello endpoint called");
    
    // Return a success status with optional data + message
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "version": "1.0.0" }))
        .message("Service started successfully")
}

// Endpoint that deliberately applies timeout to test timeout middleware
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn timeout_test_handler(
    State(_state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    let env: &'static EnvironmentVariables = EnvironmentVariables::instance();
    let timeout_seconds: u64 = env.default_timeout_seconds;
    
    info!("Testing timeout: sleeping for {} seconds (timeout is set to {} seconds)", 
          timeout_seconds + 2, timeout_seconds);
    
    // Sleep longer than the configured timeout to trigger timeout middleware
    tokio::time::sleep(std::time::Duration::from_secs(timeout_seconds + 2)).await;

    // This should never be reached due to timeout
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "message": "This should not be reached due to timeout" }))
        .message("Timeout test completed (this shouldn't happen)")
}

// Endpoint that deliberately returns a 500 error to test error handling
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn error_test_handler(
    State(_state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    info!("Testing deliberate 500 error");
    
    HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
        .data(json!({ 
            "error_type": "deliberate_test_error",
            "test_purpose": "Validate error handling middleware"
        }))
        .message("Deliberate 500 error for testing purposes")
}

// Endpoint that reads and processes the request body to test body size limits
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state))]
pub async fn body_size_test_handler(
    State(_state): State<AppState>,
    body: Bytes,
) -> HandlerResponse {
    let env: &'static EnvironmentVariables = EnvironmentVariables::instance();
    let max_size: usize = env.max_request_body_size;
    let body_size: usize = body.len();
    
    info!("Testing body size: received {} bytes (max allowed: {} bytes)", 
          body_size, max_size);

    // If we reach here, the body was within limits
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ 
            "received_body_size": body_size,
            "max_allowed_size": max_size,
            "body_preview": String::from_utf8_lossy(&body[..std::cmp::min(100, body_size)]).to_string(),
            "test_result": "Body size is within limits"
        }))
        .message(format!("Successfully processed body of {} bytes", body_size))
}

// Simple status endpoint (keeping the original hello functionality but as a separate endpoint)
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn status_handler(
    State(_state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    info!("Status endpoint called");
    
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ 
            "version": "1.0.0",
            "status": "healthy",
            "environment": EnvironmentVariables::instance().environment.as_ref()
        }))
        .message("API is running successfully")
}

// Endpoint that deliberately returns a 404 error to test not found handling
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn not_found_test_handler(
    State(_state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    info!("Testing deliberate 404 not found error");
    
    HandlerResponse::new(StatusCode::NOT_FOUND)
        .data(json!({ 
            "error_type": "deliberate_test_not_found",
            "test_purpose": "Validate 404 error handling middleware",
            "resource": "test_resource_not_found"
        }))
        .message("Deliberate 404 error for testing purposes")
}

// End of file: /src/api/test/handler.rs