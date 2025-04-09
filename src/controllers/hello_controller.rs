// Start of file: /src/controllers/hello_controller.rs

/*
    * This file contains the handler (controller) logic for the "hello" endpoint.
    * It demonstrates how to respond with JSON data and how to handle the incoming body.
*/

use serde_json::json;
use axum::{http::StatusCode, extract::State, body::Bytes};

use crate::models::state::AppState;
use std::backtrace::Backtrace;
//use std::time::Duration;
//use tokio::time::sleep;

use crate::models::response_format::StructuredResponse;

/*
    * We add an instrumentation attribute here for structured logs/traces.
*/
#[tracing::instrument(fields(backtrace = ?Backtrace::capture()))]
pub async fn hello_handler(
    State(_state): State<AppState>,
    _body: Bytes,
) -> StructuredResponse {
    //sleep(Duration::from_secs(10)).await;

    StructuredResponse::new(StatusCode::OK)
        .data(json!({ "version": "1.0.0" }))
        .message("Service started successfully")
}
 
// End of file: /src/controllers/hello_controller.rs
