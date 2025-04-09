// Start of file: /src/middlewares/response_wrapper.rs

/*
    * This middleware collects the response body, attempts to parse it into JSON,
    * then wraps it in a universal JSON structure (`ResponseFormat`).
*/

use serde_json::Value;
use axum::{
    body::Body,
    http::{
        Request, 
        Response,
        Extensions,
        response::Parts,
        header::CONTENT_TYPE
    },
    middleware::Next
};
use std::convert::Infallible;
use tracing::{error, info};
use chrono::Utc;

use crate::{
    models::response_format::{ResponseFormat, StructuredResponse},
    utils::utils::to_two_space_indented_json
};

// ======================
// Helper Functions
// ======================

fn create_default_status_message(parts: &Parts) -> String {
    parts.status
        .canonical_reason()
        .unwrap_or("UNKNOWN STATUS")
        .to_string()
}

fn extract_response_components(response: &Response<Body>) -> (Vec<String>, Value) {
    let extensions: &Extensions = response.extensions();
    let structured_response: Option<&StructuredResponse> = extensions.get::<StructuredResponse>();
    
    match structured_response {
        Some(response) => (
            response.messages.clone(),
            response.data.clone()
        ),
        None => (Vec::new(), Value::Null)
    }
}

fn log_formatted_response(wrapped: &ResponseFormat) {
    match to_two_space_indented_json(wrapped) {
        Ok(spaced_json) => info!("\nFinal response:\n{}", spaced_json),
        Err(err) => error!("Failed to format response JSON: {:?}", err)
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

// ======================
// Main Middleware Logic
// ======================

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
    let formatted_status: String = (&default_status).to_uppercase().replace(' ', "_");
    
    let wrapped: ResponseFormat = ResponseFormat {
        status: formatted_status,
        code: parts.status.as_u16(),
        data,
        messages,
        date: Utc::now().to_rfc3339()
    };
    
    // Logging
    log_formatted_response(&wrapped);
    
    // Build final response
    Ok(build_final_response(parts, &wrapped))
}

// End of file: /src/middlewares/response_wrapper.rs