// Start of file: /src/utils/response_handler/response_handler.rs

/*
Encapsulates the unified response system:
1) A `HandlerResponse` that can be returned from handlers
2) A `response_wrapper` middleware that transforms any response into a standard JSON shape.
*/

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
// We reuse a utility function to pretty-format JSON logs
use crate::utils::utils::to_two_space_indented_json;

/* ------------------------------------------------------------------------
   RESPONSE FORMAT & STRUCTURES
   ------------------------------------------------------------------------ */

// The final JSON structure returned to the client
#[derive(Serialize, Deserialize)]
pub struct ResponseFormat {
    pub status: String,          // e.g. "OK", "NOT_FOUND", "INTERNAL_SERVER_ERROR"
    pub code: u16,               // the numeric HTTP status code
    pub data: serde_json::Value, // any JSON data
    pub messages: Vec<String>,   // any informational messages
    pub date: String,            // timestamp
}

// A convenience struct that can be returned from handlers
#[derive(Debug, Clone)]
pub struct HandlerResponse {
    pub status_code: StatusCode,
    pub data: serde_json::Value,
    pub messages: Vec<String>,
}

impl HandlerResponse {
    // Initialize with a status code, defaulting data = null, messages = []
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            data: serde_json::Value::Null,
            messages: Vec::new(),
        }
    }

    // Add a JSON data object
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    // Add a message string
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.messages.push(message.into());
        self
    }
}

// Converting HandlerResponse into an Axum-compatible response
impl IntoResponse for HandlerResponse {
    fn into_response(self) -> axum::response::Response {
        // Create response with the correct status code
        let mut response: Response<Body> = Json(json!({
            "data": self.data,
            "messages": self.messages
        })).into_response();
        
        // Set the correct status code
        *response.status_mut() = self.status_code;
        
        // Insert the actual HandlerResponse into the response extensions
        // so the middleware can read it
        response.extensions_mut().insert(self);
        response
    }
}

/* ------------------------------------------------------------------------
   MIDDLEWARE: response_wrapper
   ------------------------------------------------------------------------ */

fn create_default_status_message(parts: &Parts) -> String {
    parts.status
        .canonical_reason()
        .unwrap_or("UNKNOWN STATUS")
        .to_string()
}

// Extract the messages and data from our HandlerResponse in extensions
fn extract_response_components(response: &Response<Body>) -> (Vec<String>, Value) {
    let extensions: &Extensions = response.extensions();
    let structured_response: Option<&HandlerResponse> = extensions.get::<HandlerResponse>();

    match structured_response {
        Some(r) => (r.messages.clone(), r.data.clone()),
        None => (Vec::new(), Value::Null),
    }
}

// Logs the final response in a nicely-indented JSON form
fn log_formatted_response(wrapped: &ResponseFormat) {
    match to_two_space_indented_json(wrapped) {
        Ok(spaced_json) => info!("\nFinal response:\n{}", spaced_json),
        Err(err) => error!("Failed to format response JSON: {:?}", err),
    }
}

// Builds the final Axum Response, forcing JSON
fn build_final_response(parts: Parts, wrapped: &ResponseFormat) -> Response<Body> {
    let json_body: Vec<u8> = serde_json::to_vec(wrapped).unwrap_or_else(|_| b"{}".to_vec());
    let mut new_parts: Parts = parts;

    // Force the Content-Type to JSON
    new_parts.headers.insert(
        CONTENT_TYPE,
        "application/json".parse().unwrap()
    );

    Response::from_parts(new_parts, Body::from(json_body))
}

// The main middleware that wraps every response in ResponseFormat
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    // Run the next service (handler or next layer)
    let response: Response<Body> = next.run(req).await;

    // Extract the HandlerResponse fields (messages, data)
    let (messages, data) = extract_response_components(&response);

    // Deconstruct the response into parts
    let (parts, _) = response.into_parts();

    // Build the final top-level JSON
    let default_status: String = create_default_status_message(&parts);
    let formatted_status: String = default_status.to_uppercase().replace(' ', "_");

    let wrapped: ResponseFormat = ResponseFormat {
        status: formatted_status,
        code: parts.status.as_u16(),
        data,
        messages,
        date: Utc::now().to_rfc3339(),
    };

    // Log the final JSON structure
    log_formatted_response(&wrapped);

    // Convert parts + wrapped data into a final Response
    Ok(build_final_response(parts, &wrapped))
}

// End of file: /src/utils/response_handler/response_handler.rs
