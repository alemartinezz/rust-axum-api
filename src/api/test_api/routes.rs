// Test API route definitions

use axum::{
    routing::{get, post},
    Router,
};

use crate::config::state::AppState;
use super::handler;

/// Creates router with all test endpoints for middleware validation
pub fn test_api_routes() -> Router<AppState> {
    Router::new()
        .route("/hello", get(handler::hello_handler))
        .route("/status", get(handler::status_handler))
        // Tests default 30-second timeout
        .route("/timeout", get(handler::timeout_test_handler))
        // To override timeout for specific endpoint:
        // .layer(TimeoutLayer::new(Duration::from_secs(60)))
        .route("/error", get(handler::error_test_handler))
        .route("/not-found", get(handler::not_found_test_handler))
        // Tests default 1MB body size limit
        .route("/body-size", post(handler::body_size_test_handler))
        // To override body size limit for specific endpoint:
        // .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
}