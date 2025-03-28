// Start of file: src/routes.rs

use axum::{
    Router,
    routing::get,
    middleware,
};
use crate::controllers::ping::ping_handler;
use crate::middleware::response_formatter::response_formatter;

pub fn register_routes() -> Router {
    Router::new()
        .route("/ping", get(ping_handler))
        .layer(middleware::from_fn(response_formatter))
}

// End of file: src/routes.rs
