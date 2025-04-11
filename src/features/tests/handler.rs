// Start of file: /src/features/tests/handler.rs

use std::backtrace::Backtrace;
use axum::{
    extract::State,
    body::Bytes,
    http::StatusCode
};
use tracing::instrument;
use crate::config::state::AppState;
use crate::shared::response_handler::HandlerResponse;

/*
    * Handler that sleeps 5s. If your `default_timeout_seconds` < 5,
    * requests to /timeout will trigger 408 (Request Timeout).
*/
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state))]
pub async fn timeout_handler(
    State(_state): State<AppState>,
) -> HandlerResponse {
    // Sleep 5 seconds
    tokio::time::sleep(std::time::Duration::from_secs(90)).await;

    HandlerResponse::new(StatusCode::OK)
        .message("Waited 5 seconds; no timeout if default_timeout_seconds >= 5")
}

/*
    * Handler that consumes the entire request body.
    * If it exceeds `max_request_body_size`, Axum returns 413 automatically.
*/
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(body))]
pub async fn big_payload_handler(
    State(_state): State<AppState>,
    body: Bytes,
) -> HandlerResponse {
    let size: usize = body.len();

    HandlerResponse::new(StatusCode::OK)
        .message(format!("Received {size} bytes successfully."))
}

/*
    * Handler that deliberately panics to produce a 500 error.
*/
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state))]
pub async fn error_500_handler(
    State(_state): State<AppState>,
) -> HandlerResponse {
    panic!();
    // Code here won't be reached.
}

// Start of file: /src/features/tests/handler.rs
