// Test handlers for middleware validation

use serde_json::json;
use axum::{http::StatusCode, extract::State, body::Bytes};
use std::backtrace::Backtrace;

use crate::config::state::AppState;
use crate::utils::response_handler::HandlerResponse;
use tracing::{instrument, info};

/// Basic hello endpoint with version information
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn hello_handler(
    State(_state): State<AppState>,
    _body: Bytes, // Forces body reading and triggers size limits
) -> HandlerResponse {
    info!("Hello endpoint called");
    
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "version": "1.0.0" }))
        .message("Service started successfully")
}

/// Endpoint that sleeps longer than timeout to test timeout middleware
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(state, _body))]
pub async fn timeout_test_handler(
    State(state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    let timeout_seconds = state.environment.default_timeout_seconds;
    
    info!("Testing timeout: sleeping for {} seconds (timeout is set to {} seconds)", 
          timeout_seconds + 2, timeout_seconds);
    
    // Sleep beyond configured timeout to trigger middleware
    tokio::time::sleep(std::time::Duration::from_secs(timeout_seconds + 2)).await;

    // Should never be reached due to timeout
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "message": "This should not be reached due to timeout" }))
        .message("Timeout test completed (this shouldn't happen)")
}

/// Returns 500 error to test error handling middleware
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

/// Tests body size limits by processing request body
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(state))]
pub async fn body_size_test_handler(
    State(state): State<AppState>,
    body: Bytes,
) -> HandlerResponse {
    let max_size = state.environment.max_request_body_size;
    let body_size = body.len();
    
    info!("Testing body size: received {} bytes (max allowed: {} bytes)", 
          body_size, max_size);

    HandlerResponse::new(StatusCode::OK)
        .data(json!({ 
            "received_body_size": body_size,
            "max_allowed_size": max_size,
            "body_preview": String::from_utf8_lossy(&body[..std::cmp::min(100, body_size)]).to_string(),
            "test_result": "Body size is within limits"
        }))
        .message(format!("Successfully processed body of {} bytes", body_size))
}

/// Returns API status and health information
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(state, _body))]
pub async fn status_handler(
    State(state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    info!("Status endpoint called");
    
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ 
            "version": "1.0.0",
            "status": "healthy",
            "environment": state.environment.environment.as_ref()
        }))
        .message("API is running successfully")
}

/// Returns 404 error to test not found handling middleware
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