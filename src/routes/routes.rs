// Start of file: src/routes/routes.rs

use axum::{
    Router,
    routing::get,
    middleware::from_fn,
};
use crate::controllers::ping::ping_handler;
use crate::middleware::response_formatter::response_formatter;
use crate::routes::fallback_handler::fallback_handler;

pub fn register_routes() -> Router {
    Router::new()
        // Example route
        .route("/ping", get(ping_handler))
        // Example "response formatter" middleware
        .layer(from_fn(response_formatter))
        // Fallback (404)
        .fallback(fallback_handler)
}

// End of file: src/routes/routes.rs
