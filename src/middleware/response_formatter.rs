// Start of file: src/middleware/response_formatter.rs
use axum::{
    body::{Body, BoxBody, boxed},
    http::{Request, Response},
    middleware::Next
};
use hyper::body::to_bytes;
use serde_json::Value;
use crate::models::response::ResponseFormat;

pub async fn response_formatter<B>(
    req: Request<B>,
    next: Next<B>,
) -> Response<BoxBody>
where
    B: Send + 'static,
{
    let response = next.run(req).await;
    // Extraemos el header sin mover la respuesta.
    let header_value = response
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("")
        .to_owned();

    if header_value.starts_with("application/json") {
        // Consumimos `response` y separamos sus partes.
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body).await.unwrap_or_default();
        if let Ok(original_data) = serde_json::from_slice::<Value>(&bytes) {
            let unified = ResponseFormat {
                status: parts
                    .status
                    .canonical_reason()
                    .unwrap_or("Unknown")
                    .to_string(),
                code: parts.status.as_u16(),
                data: original_data,
                messages: vec![],
                errors: vec![],
            };
            let new_body = serde_json::to_vec(&unified).unwrap_or_default();
            return Response::builder()
                .status(parts.status)
                .header("content-type", "application/json")
                .body(boxed(Body::from(new_body)))
                .unwrap();
        } else {
            // Si falla el parseo, se retorna una respuesta de fallback.
            return Response::builder()
                .status(parts.status)
                .header("content-type", "application/json")
                .body(boxed(Body::from(bytes)))
                .unwrap();
        }
    } else {
        // Rama "else": se retorna la respuesta original transformada a BoxBody.
        return response.map(boxed);
    }
}

// End of file: src/middleware/response_formatter.rs