// Start of file: /src/controllers/hello_controller.rs

use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

pub async fn hello_handler() -> (StatusCode, Json<serde_json::Value>) {
    let body: Value = json!({ "message": "Hello from Axum!" });
    
    //tokio::time::sleep(std::time::Duration::from_secs(7)).await;
    
    (StatusCode::OK, Json(body))
}

// End of file: /src/controllers/hello_controller.rs
