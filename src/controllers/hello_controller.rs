// Start of file: /src/controllers/hello_controller.rs

use axum::{http::StatusCode, Json, extract::State};
use serde_json::{json, Value};
use std::backtrace::Backtrace;
use crate::models::state::AppState;

#[tracing::instrument(fields(backtrace = ?Backtrace::capture()))]
pub async fn hello_handler(
    State(_state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    let body = json!({ "message": "Hello from Axummmm!" });
    (StatusCode::OK, Json(body))
}

// End of file: /src/controllers/hello_controller.rs
