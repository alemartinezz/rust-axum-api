// Start of file: /src/controllers/hello_controller.rs

/*
    * This file contains the handler (controller) logic for the "hello" endpoint.
    * It demonstrates how to respond with JSON data and how to handle the incoming body.
*/

use serde_json::{json, Value};
use axum::{http::StatusCode, Json, extract::State, body::Bytes};

use crate::models::state::AppState;
 
/*
    * We add an instrumentation attribute here for structured logs/traces.
*/
#[tracing::instrument]
pub async fn hello_handler(
    State(_state): State<AppState>,
    // We explicitly receive the body (even for GET) to show how to parse or ignore it.
    _body: Bytes,
) -> (StatusCode, Json<Value>) {
    let body: Value = json!({ "message": "Hello from Axummmm!" });

    // Return a tuple (StatusCode, Json), which axum interprets as an HTTP response.
    (StatusCode::OK, Json(body))
}
 
// End of file: /src/controllers/hello_controller.rs
