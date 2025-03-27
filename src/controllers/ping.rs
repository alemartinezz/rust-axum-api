use axum::{Json, http::StatusCode};
use serde_json::json;

// Este handler devuelve un JSON "crudo".
// El middleware de respuesta lo envolverá en el formato unificado.
pub async fn ping_handler() -> (StatusCode, Json<serde_json::Value>) {
    let data = json!({"message": "pong"});
    (StatusCode::OK, Json(data))
}
