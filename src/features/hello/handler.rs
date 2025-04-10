// Start of file: /src/features/hello/handler.rs

/*
    * This file contains the handler logic for the "hello" endpoint.
    * It demonstrates how to respond with JSON data and how to handle the incoming body.
*/

use serde_json::json;
use axum::{http::StatusCode, extract::State, body::Bytes};
use std::backtrace::Backtrace;

use crate::config::state::AppState;
use crate::shared::response_handler::HandlerResponse;

#[tracing::instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn hello_handler(
    State(_state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    // Example: Simulate a delay
    // tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "version": "1.0.0" }))
        .message("Service started successfully")
}

// End of file: /src/features/hello/handler.rs
