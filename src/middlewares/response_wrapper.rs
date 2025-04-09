// Start of file: /src/middlewares/response_wrapper.rs

/*
    * This middleware collects the response body, attempts to parse it into JSON,
    * then wraps it in a universal JSON structure (`ResponseFormat`).
*/
use std::{convert::Infallible, time::Instant};
use serde_json::{
    from_slice,
    Value,
};
use axum::{
    body::{Body, Bytes},
    http::{Request, Response, StatusCode, HeaderMap},
    middleware::Next,
};
use tracing::{error, warn};
use chrono::Utc;
use http_body_util::BodyExt;

use crate::models::response_format::ResponseFormat;
use crate::utils::utils::log_json;

/*
    * Converts raw bytes to JSON. If content type isn't JSON, treats body as text.
*/
fn body_to_json(raw: &[u8], headers: &HeaderMap) -> Value {
    // Check if response claims to be JSON
    let is_json: bool = headers
        .get(axum::http::header::CONTENT_TYPE)
        .map(|ct| ct.to_str().unwrap_or("").starts_with("application/json"))
        .unwrap_or(false);

    if raw.is_empty() {
        warn!("Response body is empty; defaulting to JSON null");
        Value::Null
    } else if is_json {
        // Attempt JSON parse for declared JSON content
        from_slice(raw).unwrap_or_else(|err| {
            warn!("Failed to parse JSON body: {err}");
            Value::Null
        })
    } else {
        // Treat as text if possible
        match std::str::from_utf8(raw) {
            Ok(text) => Value::String(text.to_owned()),
            Err(_) => {
                warn!("Response body is not valid UTF-8; defaulting to null");
                Value::Null
            }
        }
    }
}

/*
    * Extracts the start time from the request extensions or defaults to now.
*/
fn extract_start_time(req: &Request<Body>) -> Instant {
    req.extensions()
        .get::<Instant>()
        .copied()
        .unwrap_or_else(Instant::now)
}

/*
    * Collects the entire response body into `Bytes`.
    * Returns `(parts, collected_bytes)` on success or a `Response<Body>` on failure.
*/
async fn collect_response_body(
    response: Response<Body>,
) -> Result<(axum::http::response::Parts, Bytes), Response<Body>> {
    let (mut parts, body) = response.into_parts();
    
    match body.collect().await {
        Ok(collected) => {
            let raw_bytes: Bytes = collected.to_bytes();
            Ok((parts, raw_bytes))
        }
        Err(err) => {
            error!("Failed to collect response body: {err}");
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;

            let error_response: Response<Body> = Response::from_parts(
                parts,
                Body::from(r#"{"error": "Failed to read body"}"#),
            );
            Err(error_response)
        }
    }
}

/*
    * Builds our universal `ResponseFormat` object to standardize response output.
*/
fn build_response_format(
    parts: &axum::http::response::Parts,
    parsed_json: Value,
    start_time: Instant,
) -> ResponseFormat {
    let reason: String = parts
        .status
        .canonical_reason()
        .unwrap_or("UNKNOWN")
        .to_uppercase()
        .replace(' ', "_");

    let mut messages: Vec<String> = vec![];
    
    // Capture plain text errors into messages
    if let Value::String(message) = &parsed_json {
        messages.push(message.clone());
    }

    let duration_ms: u128 = start_time.elapsed().as_millis();
    let current_utc_date: String = Utc::now().to_rfc3339();

    ResponseFormat {
        status: reason,
        code: parts.status.as_u16(),
        data: if parsed_json.is_string() {
            Value::Null // Move text messages to 'messages' field
        } else {
            parsed_json
        },
        messages,
        time: format!("{} ms", duration_ms),
        date: current_utc_date,
    }
}

/*
    * Constructs the final HTTP response from the wrapped `ResponseFormat`.
*/
fn build_http_response(
    mut parts: axum::http::response::Parts,
    wrapped: &ResponseFormat,
) -> Response<Body> {
    let new_body: Vec<u8> = match serde_json::to_vec(wrapped) {
        Ok(json) => json,
        Err(err) => {
            error!("Failed to serialize wrapped response: {err}");
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            b"{}".to_vec()
        }
    };

    parts.headers.remove(axum::http::header::CONTENT_LENGTH);
    parts.headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    Response::from_parts(parts, Body::from(new_body))
}

/*
    * This middleware wraps each response in a uniform JSON structure.
*/
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let start_time: Instant = extract_start_time(&req);
    let response: Response<Body> = next.run(req).await;

    let (parts, raw_bytes) = match collect_response_body(response).await {
        Ok(ok) => ok,
        Err(err_response) => return Ok(err_response),
    };

    // Parse body considering content type
    let parsed_json: Value = body_to_json(&raw_bytes, &parts.headers);

    // Build and log response
    let wrapped: ResponseFormat = build_response_format(&parts, parsed_json, start_time);
    log_json(&wrapped);

    Ok(build_http_response(parts, &wrapped))
}

// End of file: /src/middlewares/response_wrapper.rs
