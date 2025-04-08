// Start of file: /src/controllers/hello_controller.rs

use axum::{http::StatusCode, Json, extract::State, body::Bytes};
use serde_json::{json, Value};
use crate::models::state::AppState;

#[tracing::instrument]
pub async fn hello_handler(
    State(_state): State<AppState>,
    // Explicitly consume body even for GET requests
    _body: Bytes,
) -> (StatusCode, Json<Value>) {
    let body: Value = json!({ "message": "Hello from Axummmm!" });
    (StatusCode::OK, Json(body))
}

// End of file: /src/controllers/hello_controller.rs
