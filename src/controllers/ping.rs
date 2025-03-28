// Start of file: src/controllers/ping.rs

use axum::{http::StatusCode, Json};
use serde_json::json;

pub async fn ping_handler() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({ "message": "pong" })))
}

// End of file: src/controllers/ping.rs