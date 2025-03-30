// Start of file: /src/middlewares/response_wrapper.rs

use std::convert::Infallible;
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use http_body_util::BodyExt;
use serde_json::{from_slice, to_vec, Value};

use crate::models::response_format::ResponseFormat;

pub async fn response_wrapper(
    req: Request<Body>,
    next: Next
) -> Result<Response<Body>, Infallible> {
    // 1) Call the inner handler or next middleware
    let response = next.run(req).await;

    // 2) Split parts so we can examine the body
    let (mut parts, body) = response.into_parts();

    // 3) Collect the entire body
    let collected = match body.collect().await {
        Ok(data) => data,
        Err(_) => {
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            // Return the rebuilt response inside Ok(...) so it remains `Result<_, Infallible>`
            return Ok(Response::from_parts(parts, Body::from(r#"{"error": "Failed to read body"}"#)));
        }
    };

    // 4) Convert to raw bytes
    let raw_bytes = collected.to_bytes();

    // 5) Attempt to parse as JSON
    let parsed_json: Value = from_slice(&raw_bytes).unwrap_or_else(|_| Value::Null);

    // 6) Build a reason string (e.g. 200 -> "OK", 404 -> "NOT_FOUND", etc.)
    let reason = parts
        .status
        .canonical_reason()
        .unwrap_or("UNKNOWN")
        .to_uppercase()
        .replace(' ', "_");

    // 7) Wrap in your universal ResponseFormat
    let wrapped = ResponseFormat {
        status: reason,
        code: parts.status.as_u16(),
        data: parsed_json,
        messages: vec![],
        errors: vec![],
    };

    // 8) Encode the new JSON body
    let new_body = match to_vec(&wrapped) {
        Ok(json) => json,
        Err(_) => {
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            b"{}".to_vec()
        }
    };

    // 9) Update headers
    parts.headers.remove(axum::http::header::CONTENT_LENGTH);
    parts.headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    // Finally, wrap the response in `Ok(...)` to satisfy `Result<_, Infallible>`
    Ok(Response::from_parts(parts, Body::from(new_body)))
}

// End of file: /src/middlewares/response_wrapper.rs
