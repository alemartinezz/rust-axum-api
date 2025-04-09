// Start of file: /src/middlewares/response_wrapper.rs

/*
    * This middleware collects the response body, attempts to parse it into JSON,
    * then wraps it in a universal JSON structure (`ResponseFormat`).
*/
use serde_json::{
    from_slice,
    Value,
};
use axum::{
    body::{
        Body,
        Bytes
    },
    http::{
        Request,
        Response,
        StatusCode,
        HeaderMap,
        HeaderValue,
        response::Parts,header::{
            CONTENT_TYPE,
            CONTENT_LENGTH
        }},
    middleware::Next
};
use std::convert::Infallible;
use http_body_util::BodyExt;
use tracing::{error, info, warn};
use chrono::Utc;

use crate::models::response_format::ResponseFormat;

/*
    * Converts raw bytes to JSON. If content type isn't JSON, treats body as text.
*/
fn body_to_json(raw: &[u8], headers: &HeaderMap) -> Value {
    // Check if response claims to be JSON
    let is_json: bool = headers
        .get(CONTENT_TYPE)
        .map(|ct: &HeaderValue | ct.to_str().unwrap_or("").starts_with("application/json"))
        .unwrap_or(false);

    if raw.is_empty() {
        warn!("Response body is empty; defaulting to JSON null");
        Value::Null
    } else if is_json {
        // Attempt JSON parse for declared JSON content
        from_slice(raw).unwrap_or_else(|err: serde_json::Error| {
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
    * Collects the entire response body into `Bytes`.
    * Returns `(parts, collected_bytes)` on success or a `Response<Body>` on failure.
*/
async fn collect_response_body(
    response: Response<Body>,
) -> Result<(Parts, Bytes), Response<Body>> {
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
    parts: &Parts,
    parsed_json: Value,
) -> ResponseFormat {
    // Get the canonical reason phrase (e.g. "Switching Protocols")
    let default_message: String = parts.status
        .canonical_reason()
        .unwrap_or("UNKNOWN STATUS")
        .to_string();

    let mut messages: Vec<String> = vec![];
    
    // Capture explicit messages from response body
    if let Value::String(message) = &parsed_json {
        messages.push(message.clone());
    }

    let current_utc_date: String = Utc::now().to_rfc3339();

    ResponseFormat {
        status: default_message.to_uppercase().replace(' ', "_"),
        code: parts.status.as_u16(),
        data: if parsed_json.is_string() {
            Value::Null
        } else {
            parsed_json
        },
        messages,
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

    parts.headers.remove(CONTENT_LENGTH);
    parts.headers.insert(
        CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    Response::from_parts(parts, Body::from(new_body))
}

/*
    * This middleware wraps each response in a uniform JSON structure.
*/
// In /src/middlewares/response_wrapper.rs

pub async fn response_wrapper(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let response: Response<Body> = next.run(req).await;

    let (parts, raw_bytes) = match collect_response_body(response).await {
        Ok(ok) => ok,
        Err(err_response) => return Ok(err_response),
    };

    // Parse body considering content type
    let parsed_json: Value = body_to_json(&raw_bytes, &parts.headers);

    // Build response
    let wrapped: ResponseFormat = build_response_format(&parts, parsed_json);
    
    // Format and log directly here
    match crate::utils::utils::to_two_space_indented_json(&wrapped) {
        Ok(spaced_json) => {
            info!("\nFinal response:\n{}", spaced_json);
        }
        Err(err) => {
            error!("Failed to format response JSON: {:?}", err);
        }
    }

    Ok(build_http_response(parts, &wrapped))
}

// End of file: /src/middlewares/response_wrapper.rs
