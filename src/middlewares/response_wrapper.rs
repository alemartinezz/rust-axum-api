// Start of file: /src/middlewares/response_wrapper.rs

use axum::{
    body::{Body, Bytes},
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use chrono::Utc;
use http_body_util::BodyExt;
use serde::Serialize;
use serde_json::{
    from_slice,
    ser::{PrettyFormatter, Serializer},
    Value,
};
use std::{convert::Infallible, time::Instant};
use tracing::{error, info, warn};

use crate::models::response_format::ResponseFormat;

/// Converts raw bytes to JSON. If conversion fails or the bytes are empty,
/// returns `Value::Null`.
fn body_to_json(raw: &[u8]) -> Value {
    if raw.is_empty() {
        warn!("Response body is empty; defaulting to JSON null");
        Value::Null
    } else {
        from_slice(raw).unwrap_or_else(|err: serde_json::Error| {
            warn!("Failed to parse response body as JSON: {err}");
            Value::Null
        })
    }
}

/// Convert any `Serialize` type into a two space‐indented JSON string.
fn to_two_space_indented_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let mut writer: Vec<u8> = Vec::new();
    let formatter: PrettyFormatter<'_> = PrettyFormatter::with_indent(b"  ");
    let mut ser: Serializer<&mut Vec<u8>, PrettyFormatter<'_>> = Serializer::with_formatter(&mut writer, formatter);

    value.serialize(&mut ser)?;

    Ok(String::from_utf8(writer).expect("should be valid UTF-8"))
}

/// Extracts the start time from the request extensions.
/// If it's missing for some reason, defaults to `Instant::now()`.
fn extract_start_time(req: &Request<Body>) -> Instant {
    req.extensions()
        .get::<Instant>()
        .copied()
        .unwrap_or_else(Instant::now)
}

/// Collects the entire response body into `Bytes`. Returns a tuple
/// of `(parts, collected_bytes)` on success, or a `Response<Body>`
/// for immediate error return on failure.
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

/// Builds our universal `ResponseFormat`.
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
    
    if parts.status == StatusCode::REQUEST_TIMEOUT {
        messages.push("The request timed out after 10 seconds.".to_owned());
    }

    let duration_ms: u128 = start_time.elapsed().as_millis();
    let current_utc_date: String = Utc::now().to_rfc3339();

    ResponseFormat {
        status: reason,
        code: parts.status.as_u16(),
        data: parsed_json,
        messages,
        time: format!("{} ms", duration_ms),
        date: current_utc_date,
    }
}

/// Logs the `ResponseFormat` in two space‐indented JSON format.
fn log_response(wrapped: &ResponseFormat) {
    match to_two_space_indented_json(wrapped) {
        Ok(spaced_json) => {
            info!("\n{}", spaced_json);
        }
        Err(err) => {
            error!("Could not format response as two space‐indented JSON: {err}");
        }
    }
}

/// Constructs the final HTTP response from the given `parts` and
/// `ResponseFormat`, ensuring a JSON body and appropriate headers.
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

    parts
        .headers
        .remove(axum::http::header::CONTENT_LENGTH);
    parts.headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    Response::from_parts(parts, Body::from(new_body))
}

/// Middleware that wraps the response in a universal JSON format.
pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let start_time: Instant = extract_start_time(&req);
    let response: Response<Body> = next.run(req).await;

    let (parts, raw_bytes) = match collect_response_body(response).await {
        Ok(ok) => ok,
        Err(err_response) => {
            return Ok(err_response);
        }
    };

    let parsed_json: Value = body_to_json(&raw_bytes);
    let wrapped: ResponseFormat = build_response_format(&parts, parsed_json, start_time);

    log_response(&wrapped);
    let final_response: Response<Body> = build_http_response(parts, &wrapped);

    Ok(final_response)
}

// End of file: /src/middlewares/response_wrapper.rs