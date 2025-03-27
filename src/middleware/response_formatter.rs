use axum::{
    body::{Body, Bytes, BoxBody, boxed},
    http::{Response, StatusCode},
};
use futures::future::BoxFuture;
use futures::FutureExt;
use hyper::body;
use serde_json::Value;
use crate::models::response::ResponseFormat;
use std::task::{Context, Poll};
use tower::Layer;
use tower::Service;
use std::convert::Infallible;

#[derive(Clone, Default)]
pub struct ResponseFormatterLayer;

impl<S> Layer<S> for ResponseFormatterLayer {
    type Service = ResponseFormatterMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ResponseFormatterMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ResponseFormatterMiddleware<S> {
    inner: S,
}

impl<S, ReqBody> Service<axum::http::Request<ReqBody>> for ResponseFormatterMiddleware<S>
where
    // Cambiamos el bound para esperar Response<BoxBody> en lugar de Response<Body>
    S: Service<axum::http::Request<ReqBody>, Response = Response<BoxBody>, Error = Infallible> 
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        async move {
            let mut response = inner.call(req).await?;

            // Verifica que el Content-Type sea JSON.
            let content_type = response.headers().get("content-type").cloned();
            if let Some(ct) = content_type {
                if ct.to_str().unwrap_or("").starts_with("application/json") {
                    // Lee el body completo.
                    let body_bytes = body::to_bytes(response.body_mut()).await.unwrap_or_else(|_| Bytes::new());
                    // Intenta parsear el body como JSON.
                    if let Ok(original_data) = serde_json::from_slice::<Value>(&body_bytes) {
                        // Crea la respuesta unificada.
                        let unified = ResponseFormat {
                            status: StatusCode::from_u16(response.status().as_u16())
                                .unwrap()
                                .canonical_reason()
                                .unwrap_or("Unknown")
                                .to_string(),
                            code: response.status().as_u16(),
                            data: original_data,
                            messages: vec![],
                            errors: vec![],
                        };
                        let unified_body = serde_json::to_vec(&unified).unwrap_or_default();
                        // Crea directamente el body a partir del Vec<u8>
                        let new_body = Body::from(unified_body);

                        let new_response = Response::builder()
                            .status(response.status())
                            .header("content-type", "application/json")
                            .body(boxed(new_body))
                            .unwrap();

                        // Copia otros headers si es necesario.
                        let mut new_response = new_response;
                        *new_response.headers_mut() = response.headers().clone();
                        return Ok(new_response);
                    }
                }
            }
            // Si no se cumple la condición, devolvemos la respuesta original (ya con BoxBody).
            Ok(response)
        }
        .boxed()
    }
}
