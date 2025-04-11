// Start of file: /src/features/tests/handler.rs

use axum::{routing::{get, post}, Router};
use crate::config::state::AppState;
use super::handler::{timeout_handler, big_payload_handler, error_500_handler};

/*
    * Aggregates the three test endpoints into one Router:
    *  - GET /timeout => sleeps 5s → 408 if default_timeout_seconds < 5
    *  - POST /big-payload => returns 413 if payload > max_request_body_size
    *  - GET /error-500 => panic => 500
*/
pub fn tests_routes() -> Router<AppState> {
    Router::new()
        .route("/timeout", get(timeout_handler))
        .route("/big-payload", post(big_payload_handler))
        .route("/error-500", get(error_500_handler))
}

// End of file: /src/features/tests/handler.rs
