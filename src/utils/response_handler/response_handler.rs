// Unified response system for consistent API responses
// Provides HandlerResponse struct and middleware for standardizing all responses

use axum::{
    body::Body,
    http::{
        header::CONTENT_TYPE, Request, Response, 
        response::Parts, StatusCode, Extensions
    },
    Json,
    middleware::Next,
    response::IntoResponse,
};
use chrono::Utc;
use tracing::{error, info};
use std::convert::Infallible;
use serde_json::{json, Value};
use serde::{Serialize, Deserialize};
use crate::utils::utils::to_two_space_indented_json;

/// Standard JSON response format for all API endpoints
#[derive(Serialize, Deserialize)]
pub struct ResponseFormat {
    pub status: String,          // HTTP status text (e.g. "OK", "NOT_FOUND")
    pub code: u16,               // HTTP status code
    pub data: serde_json::Value, // Response payload
    pub messages: Vec<String>,   // Informational messages
    pub date: String,            // ISO timestamp
}

/// Convenience struct for building responses in handlers
#[derive(Debug, Clone)]
pub struct HandlerResponse {
    pub status_code: StatusCode,
    pub data: serde_json::Value,
    pub messages: Vec<String>,
}

impl HandlerResponse {
    /// Creates a new response with specified status code
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            data: serde_json::Value::Null,
            messages: Vec::new(),
        }
    }

    /// Adds JSON data payload to the response
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Adds an informational message to the response
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.messages.push(message.into());
        self
    }
}

impl IntoResponse for HandlerResponse {
    fn into_response(self) -> axum::response::Response {
        let mut response: Response<Body> = Json(json!({
            "data": self.data,
            "messages": self.messages
        })).into_response();
        
        *response.status_mut() = self.status_code;
        
        // Store HandlerResponse in extensions for middleware processing
        response.extensions_mut().insert(self);
        response
    }
}

fn create_default_status_message(parts: &Parts) -> String {
    parts.status
        .canonical_reason()
        .unwrap_or("UNKNOWN STATUS")
        .to_string()
}

/// Extracts response data and messages from HandlerResponse extensions
fn extract_response_components(response: &Response<Body>) -> (Vec<String>, Value) {
    let extensions: &Extensions = response.extensions();
    let structured_response: Option<&HandlerResponse> = extensions.get::<HandlerResponse>();

    match structured_response {
        Some(r) => (r.messages.clone(), r.data.clone()),
        None => (Vec::new(), Value::Null),
    }
}

/// Logs the formatted response with proper JSON indentation
fn log_formatted_response(wrapped: &ResponseFormat) {
    match to_two_space_indented_json(wrapped) {
        Ok(spaced_json) => info!("\nFinal response:\n{}", spaced_json),
        Err(err) => error!("Failed to format response JSON: {:?}", err),
    }
}

/// Builds the final response with JSON content type
fn build_final_response(parts: Parts, wrapped: &ResponseFormat) -> Response<Body> {
    let json_body: Vec<u8> = serde_json::to_vec(wrapped).unwrap_or_else(|_| b"{}".to_vec());
    let mut new_parts: Parts = parts;

    new_parts.headers.insert(
        CONTENT_TYPE,
        "application/json".parse().unwrap()
    );

    Response::from_parts(new_parts, Body::from(json_body))
}

/// Middleware that wraps all responses in the standard ResponseFormat structure
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let response: Response<Body> = next.run(req).await;

    let (messages, data) = extract_response_components(&response);
    let (parts, _) = response.into_parts();

    let default_status: String = create_default_status_message(&parts);
    let formatted_status: String = default_status.to_uppercase().replace(' ', "_");

    let wrapped: ResponseFormat = ResponseFormat {
        status: formatted_status,
        code: parts.status.as_u16(),
        data,
        messages,
        date: Utc::now().to_rfc3339(),
    };

    log_formatted_response(&wrapped);

    Ok(build_final_response(parts, &wrapped))
}
