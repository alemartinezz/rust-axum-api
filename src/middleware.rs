// Start of file: /src/middleware.rs

use axum::{
    body::{Body},
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use http_body_util::BodyExt; // needed for collect()
use serde_json::{from_slice, to_vec, Value};

use crate::models::response_format::ResponseFormat;

// Middleware: Buffers the entire response, tries to parse it as JSON, and wraps it.
pub async fn wrap_in_response_format(req: Request<Body>, next: Next) -> Response<Body> {
    // 1) Call the inner handler or next middleware
    let response = next.run(req).await;
    
    // 2) Split parts so we can examine the body
    let (mut parts, body) = response.into_parts();
    
    // 3) Collect the entire body
    let collected = match body.collect().await {
        Ok(data) => data, // `Collected<Bytes>`
        Err(_) => {
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            return Response::from_parts(
                parts,
                Body::from(r#"{"error": "Failed to read body"}"#)
            );
        }
    };

    // 4) Convert to raw bytes
    let raw_bytes = collected.to_bytes();
    
    // 5) Attempt to parse as JSON
    let parsed_json: Value = from_slice(&raw_bytes).unwrap_or_else(|_| Value::Null);
    
    // -- NEW: Use HTTP reason phrase for the `status` field
    // E.g. 200 => "OK", 404 => "NOT_FOUND"
    let reason = parts
        .status
        .canonical_reason()       // e.g. Some("Not Found")
        .unwrap_or("UNKNOWN")     // fallback if missing a reason
        .to_uppercase()           // e.g. "NOT FOUND"
        .replace(' ', "_");       // => "NOT_FOUND"
    
    // 6) Wrap in your ResponseFormat
    let wrapped = ResponseFormat {
        status: reason,
        code: parts.status.as_u16(),
        data: parsed_json,
        messages: vec![],
        errors: vec![],
    };
    
    // 7) Convert the new JSON to bytes
    let new_body = match to_vec(&wrapped) {
        Ok(body) => body,
        Err(_) => {
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            b"{}".to_vec()
        }
    };

    // 8) Adjust headers to reflect new JSON body
    parts.headers.remove(axum::http::header::CONTENT_LENGTH);
    parts.headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    // Recombine into a Response<Body>
    Response::from_parts(parts, Body::from(new_body))
}

// End of file: /src/middleware.rs
