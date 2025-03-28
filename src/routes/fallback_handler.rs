// Start of file: src/routes/fallback_handler.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::models::response::ResponseFormat;
use serde_json::Value;

pub async fn fallback_handler() -> Response {
    let unified: ResponseFormat = ResponseFormat {
        status: "Not Found".to_string(),
        code: StatusCode::NOT_FOUND.as_u16(),
        data: Value::Null,
        messages: vec![],
        errors: vec!["La ruta solicitada no existe".to_string()],
    };

    (StatusCode::NOT_FOUND, Json(unified)).into_response()
}

// End of file: src/routes/fallback_handler.rs
