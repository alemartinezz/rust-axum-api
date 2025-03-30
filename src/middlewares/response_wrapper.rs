// Start of file: /src/middlewares/response_wrapper.rs

use std::convert::Infallible;
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use http_body_util::BodyExt;
use serde_json::{from_slice, to_vec, Value};
use tracing::{error, warn, info};

use crate::models::response_format::ResponseFormat;

/// Converts the raw body bytes into a JSON value. If the conversion fails,
/// returns `Value::Null` while logging a warning.
fn body_to_json(raw: &[u8]) -> Value {
    if raw.is_empty() {
        warn!("Response body is empty; defaulting to JSON null");
        Value::Null
    } else {
        from_slice(raw).unwrap_or_else(|err| {
            warn!("Failed to parse response body as JSON: {err}");
            Value::Null
        })
    }
}

/// Middleware that wraps the outgoing response body in your universal `ResponseFormat`.
/// It logs errors and gracefully handles empty bodies.
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    // Call the inner handler
    let response = next.run(req).await;
    let (mut parts, body) = response.into_parts();

    // Collect the entire body into memory
    let collected = match body.collect().await {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to collect response body: {err}");
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(Response::from_parts(
                parts,
                Body::from(r#"{"error": "Failed to read body"}"#),
            ));
        }
    };

    // Convert collected bytes into a slice and parse as JSON
    let raw_bytes = collected.to_bytes();
    let parsed_json = body_to_json(&raw_bytes);

    // Build a reason string from the status (e.g. "OK", "NOT_FOUND")
    let reason = parts
        .status
        .canonical_reason()
        .unwrap_or("UNKNOWN")
        .to_uppercase()
        .replace(' ', "_");

    info!("Wrapping response: status = {}, body length = {} bytes", reason, raw_bytes.len());

    // Wrap the parsed JSON in your universal response format
    let wrapped = ResponseFormat {
        status: reason,
        code: parts.status.as_u16(),
        data: parsed_json,
        messages: vec![],
        errors: vec![],
    };

    // Encode the new JSON body
    let new_body = match to_vec(&wrapped) {
        Ok(json) => json,
        Err(err) => {
            error!("Failed to serialize wrapped response: {err}");
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            b"{}".to_vec()
        }
    };

    // Update headers to set content type and remove outdated content-length
    parts.headers.remove(axum::http::header::CONTENT_LENGTH);
    parts.headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    Ok(Response::from_parts(parts, Body::from(new_body)))
}


// End of file: /src/middlewares/response_wrapper.rs
