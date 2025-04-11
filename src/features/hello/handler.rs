// Start of file: /src/features/hello/handler.rs

// * Demonstrates how to handle a request, read the request body, and produce a JSON response.

use serde_json::json;
use axum::{http::StatusCode, extract::State, body::Bytes};
use std::backtrace::Backtrace;

use crate::config::state::AppState;
use crate::shared::response_handler::HandlerResponse;
use tracing::instrument;

// * The hello_handler is a simple example endpoint
// ? It demonstrates returning a structured response using HandlerResponse
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn hello_handler(
    State(_state): State<AppState>,
    _body: Bytes,        // * This forces Axum to read the body, also triggers body-size limits
) -> HandlerResponse {
    // ! Example: You could simulate a delay if desired to check the timeout middleware
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // * Return a success status with optional data + message
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "version": "1.0.0" }))
        .message("Service started successfully")
}

// End of file: /src/features/hello/handler.rs
