// Start of file: /src/app.rs

use std::time::Duration;

use axum::{
    Router,
    middleware::from_fn,
};
use tower_http::timeout::TimeoutLayer;

use crate::routes::hello_route::hello_routes;
use crate::middlewares::{
    response_wrapper::response_wrapper,
    start_time::start_time_middleware,
};

pub fn create_app() -> Router {
    Router::new()
        .merge(hello_routes())

        // 1) Innermost layer: 10s request timeout
        //    If the handler doesn't finish in 10s, this returns a 408 response
        .layer(TimeoutLayer::new(Duration::from_secs(5)))

        // 2) Next layer: wrap *all* responses (including the 408) in JSON
        .layer(from_fn(response_wrapper))

        // 3) Outermost layer: track start times for logging/metrics
        .layer(from_fn(start_time_middleware))
}

// End of file: /src/app.rs