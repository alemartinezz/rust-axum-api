// Start of file: /src/models/response_format.rs

/*
    * Defines a universal JSON response format used by our custom middleware
    * to wrap responses in a consistent structure.
*/

use axum::{http::StatusCode, Json};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use axum::response::{IntoResponse, Response};

#[derive(Serialize, Deserialize)]
pub struct ResponseFormat {
    // The textual representation of HTTP status (e.g., "OK" -> "OK", "404" -> "NOT_FOUND").
    pub status: String,
    // The numeric HTTP status code (e.g., 200, 404).
    pub code: u16,
    // The actual data payload in JSON form.
    pub data: Value,
    // A list of optional messages (e.g., warnings, errors).
    pub messages: Vec<String>,
    // The UTC date/time (RFC3339) when the server responded.
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct StructuredResponse {
    pub status_code: StatusCode,
    pub data: Value,
    pub messages: Vec<String>,
}

impl StructuredResponse {
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            data: Value::Null,
            messages: Vec::new(),
        }
    }

    pub fn data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.messages.push(message.into());
        self
    }
}

impl IntoResponse for StructuredResponse {
    fn into_response(self) -> Response {
        // Store messages in response extensions
        let mut response = Json(json!({
            "data": self.data,
            "messages": self.messages
        })).into_response();
        
        response.extensions_mut().insert(self);
        response
    }
}

// End of file: /src/models/response_format.rs
