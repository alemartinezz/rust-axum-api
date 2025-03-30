// Start of file: /src/middlewares/response_wrapper.rs

use std::{
    convert::Infallible,
    time::Instant,
};
use axum::{
    body::{Body, Bytes},
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use http_body_util::{BodyExt, Collected};
use serde::Serialize;
use serde_json::{
    from_slice, ser::{PrettyFormatter, Serializer}, Value,
};
use tracing::{error, warn, info};
use chrono::Utc;

use crate::models::response_format::ResponseFormat;

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

/// Convert any `Serialize` type into a tab‐indented JSON string.
fn to_tab_indented_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let mut writer = Vec::new();
    // Use `\t` for indentation
    let formatter = PrettyFormatter::with_indent(b"\t");
    let mut ser = Serializer::with_formatter(&mut writer, formatter);
    value.serialize(&mut ser)?;
    Ok(String::from_utf8(writer).expect("should be valid UTF-8"))
}

pub async fn response_wrapper(
    req: Request<Body>,
    next: Next<>,
) -> Result<Response<Body>, Infallible> {
    // Pull out the start time from request extensions (if present).
    // If it's missing for some reason, default to "now()".
    let start_time: Instant = req
        .extensions()
        .get::<Instant>()
        .copied() // get a copy of the Instant
        .unwrap_or_else(Instant::now);

    // Call the inner handler
    let response: Response<Body> = next.run(req).await;
    let (mut parts, body) = response.into_parts();

    // Collect the entire body
    let collected: Collected<Bytes> = match body.collect().await {
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
    let raw_bytes: Bytes = collected.to_bytes();
    let parsed_json: Value = body_to_json(&raw_bytes);

    // Build a reason string from the status (e.g. "OK", "NOT_FOUND")
    let reason: String = parts
        .status
        .canonical_reason()
        .unwrap_or("UNKNOWN")
        .to_uppercase()
        .replace(' ', "_");

    // Calculate the duration in milliseconds
    let duration_ms: u128 = start_time.elapsed().as_millis();

    // Current UTC date/time in ISO 8601
    let current_utc_date: String = Utc::now().to_rfc3339();

    // Wrap the parsed JSON in your universal response format
    let wrapped: ResponseFormat = ResponseFormat {
        status: reason,
        code: parts.status.as_u16(),
        data: parsed_json,
        messages: vec![],
        errors: vec![],
        time: format!("{} ms", duration_ms),
        date: current_utc_date,
    };

    // Log the final response in a tab‐indented format
    match to_tab_indented_json(&wrapped) {
        Ok(tabbed_json) => {
            info!("\n\n{}\n", tabbed_json);
        }
        Err(err) => {
            error!("Could not format response as tab‐indented JSON: {err}");
        }
    }

    // Encode the new JSON body for the actual response
    let new_body: Vec<u8> = match serde_json::to_vec(&wrapped) {
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
