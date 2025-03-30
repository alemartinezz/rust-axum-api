// Start of file: /src/controllers/hello_controller.rs

use axum::{http::StatusCode, Json};
use serde_json::json;

pub async fn hello_handler() -> (StatusCode, Json<serde_json::Value>) {
    let body = json!({ "message": "Hello from Axum!" });
    (StatusCode::OK, Json(body))
}

// End of file: /src/controllers/hello_controller.rs
