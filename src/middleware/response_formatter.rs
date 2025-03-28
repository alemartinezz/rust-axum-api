// Start of file: src/middleware/response_formatter.rs

use axum::{
    body::{Body, to_bytes},
    http::{Request, Response},
    middleware::Next,
};
use serde_json::Value;
use crate::models::response::ResponseFormat;

pub async fn response_formatter(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Pass request down the stack
    let response: Response<Body> = next.run(req).await;

    // If the response is JSON, wrap it in a standardized format
    let header_value: String = response
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("")
        .to_owned();

    if header_value.starts_with("application/json") {
        let (parts, body) = response.into_parts();

        // Read the body (limit omitted for brevity).
        let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();


        if let Ok(original_data) = serde_json::from_slice::<Value>(&bytes) {
            // Wrap it
            let unified: ResponseFormat = ResponseFormat {
                status: parts.status.canonical_reason().unwrap_or("Unknown").to_string(),
                code: parts.status.as_u16(),
                data: original_data,
                messages: vec![],
                errors: vec![],
            };
            let new_body: Vec<u8>  = serde_json::to_vec(&unified).unwrap_or_default();

            return Response::builder()
                .status(parts.status)
                .header("content-type", "application/json")
                .body(Body::from(new_body))
                .unwrap();
        } else {
            // If it wasn't valid JSON, just pass the raw bytes back unchanged
            return Response::builder()
                .status(parts.status)
                .header("content-type", "application/json")
                .body(Body::from(bytes))
                .unwrap();
        }
    }

    response
}

// End of file: src/middleware/response_formatter.rs