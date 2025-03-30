// Start of file: /src/app.rs

use axum::{
    Router,
    middleware::from_fn,
};

use crate::routes::hello_route::hello_routes;
use crate::middlewares::{
    response_wrapper::response_wrapper,
    start_time::start_time_middleware,
};

pub fn create_app() -> Router {
    // Combine routes
    Router::new()
        .merge(hello_routes())
        // Put start_time_middleware *last* so it is the "outermost"
        .layer(from_fn(start_time_middleware))
        // Put response_wrapper *under* the timing layer
        .layer(from_fn(response_wrapper))
        
}

// End of file: /src/app.rs