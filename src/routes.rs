use axum::{Router, routing::get};
use crate::controllers::ping::ping_handler;
use crate::middleware::response_formatter::ResponseFormatterLayer;

pub fn register_routes() -> Router {
    Router::new()
        .route("/ping", get(ping_handler))
        .layer(ResponseFormatterLayer::default())
}
