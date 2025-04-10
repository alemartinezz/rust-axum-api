// Start of file: /src/shared/response_handler/response_handler.rs

/*
    * This module encapsulates everything related to our response handling:
    * - A universal `ResponseFormat` struct (and `HandlerResponse`)
    * - The `response_wrapper` middleware that uses them.
*/

use axum::{
    body::Body,
    http::{
        Request, Response, Extensions,
        response::Parts, header::CONTENT_TYPE,
        StatusCode
    },
    Json,
    middleware::Next,
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::convert::Infallible;
use tracing::{error, info};
use crate::shared::utils::to_two_space_indented_json;

/* ------------------------------------------------------------------------
   RESPONSE FORMAT & STRUCTURES
   ------------------------------------------------------------------------ */

#[derive(Serialize, Deserialize)]
pub struct ResponseFormat {
    // The textual representation of HTTP status (e.g., "OK", "NOT_FOUND").
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
pub struct HandlerResponse {
    pub status_code: StatusCode,
    pub data: Value,
    pub messages: Vec<String>,
}

impl HandlerResponse {
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

impl IntoResponse for HandlerResponse {
    fn into_response(self) -> axum::response::Response {
        // Store messages in the response extensions
        let mut response: Response<Body> = Json(json!({
            "data": self.data,
            "messages": self.messages
        }))
        .into_response();

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

fn extract_response_components(response: &Response<Body>) -> (Vec<String>, Value) {
    let extensions: &Extensions = response.extensions();
    let structured_response: Option<&HandlerResponse> = extensions.get::<HandlerResponse>();

    match structured_response {
        Some(r) => (r.messages.clone(), r.data.clone()),
        None => (Vec::new(), Value::Null),
    }
}

fn log_formatted_response(wrapped: &ResponseFormat) {
    match to_two_space_indented_json(wrapped) {
        Ok(spaced_json) => info!("\nFinal response:\n{}", spaced_json),
        Err(err) => error!("Failed to format response JSON: {:?}", err),
    }
}

fn build_final_response(parts: Parts, wrapped: &ResponseFormat) -> Response<Body> {
    let json_body: Vec<u8> = serde_json::to_vec(wrapped).unwrap_or_else(|_| b"{}".to_vec());
    let mut new_parts: Parts = parts;

    new_parts.headers.insert(
        CONTENT_TYPE,
        "application/json".parse().unwrap()
    );

    Response::from_parts(new_parts, Body::from(json_body))
}

/**
 * This middleware collects the response body, attempts to parse it into JSON,
 * then wraps it in our universal `ResponseFormat`.
 */
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let response: Response<Body> = next.run(req).await;

    // Extract components before consuming the response
    let (messages, data) = extract_response_components(&response);

    // Deconstruct response into parts
    let (parts, _) = response.into_parts();

    // Build formatted response
    let default_status: String = create_default_status_message(&parts);
    let formatted_status: String = default_status.to_uppercase().replace(' ', "_");

    let wrapped: ResponseFormat = ResponseFormat {
        status: formatted_status,
        code: parts.status.as_u16(),
        data,
        messages,
        date: Utc::now().to_rfc3339(),
    };

    // Logging
    log_formatted_response(&wrapped);

    // Build final response
    Ok(build_final_response(parts, &wrapped))
}

// End of file: /src/shared/response_handler/response_handler.rs
