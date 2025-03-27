use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use crate::models::response::ResponseFormat;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub status: StatusCode,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = ResponseFormat {
            status: self.status.canonical_reason().unwrap_or("Error").to_string(),
            code: self.status.as_u16(),
            data: Value::Null,
            messages: vec![],
            errors: vec![self.message],
        };
        (self.status, Json(body)).into_response()
    }
}
