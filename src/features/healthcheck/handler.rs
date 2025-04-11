// Start of file: /src/features/healthcheck/handler.rs

use serde_json::json;
use axum::{http::StatusCode, extract::State, body::Bytes};
use std::backtrace::Backtrace;

use crate::config::state::AppState;
use tracing::instrument;
use crate::shared::response_handler::HandlerResponse;

// * The healthcheck_handler is a simple example endpoint
// ? It demonstrates returning a structured response using HandlerResponse
#[instrument(fields(backtrace = ?Backtrace::capture()), skip(_state, _body))]
pub async fn healthcheck_handler(
    State(_state): State<AppState>,
    _body: Bytes,
) -> HandlerResponse {
    HandlerResponse::new(StatusCode::OK)
        .data(json!({ "version": env!("CARGO_PKG_VERSION") }))
        .message("Hello from Axum!")
}

// End of file: /src/features/healthcheck/handler.rs
