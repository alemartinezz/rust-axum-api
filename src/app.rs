use axum::{
    Router,
    middleware::from_fn,
};

use crate::routes::hello_route::hello_routes;
use crate::middlewares::response_wrapper::response_wrapper;

// Build the entire application router.
// This function combines all your routes and applies global middleware.
pub fn create_app() -> Router {
    // Combine routes
    Router::new()
        .merge(hello_routes())
        // Apply your universal JSON response wrapper
        .layer(from_fn(response_wrapper))
}
