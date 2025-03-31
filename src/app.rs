use std::time::Duration;

use axum::{
    Router,
    http::StatusCode,
    error_handling::HandleErrorLayer,
    middleware::from_fn,
    BoxError
};
use tower::ServiceBuilder;
use tower::timeout::TimeoutLayer;

use crate::routes::hello_route::hello_routes;
use crate::middlewares::{
    response_wrapper::response_wrapper,
    start_time::start_time_middleware,
};

/// Convert any error from the timeout (or other layers) into a valid HTTP response:
/// - If it's a timeout error (`tower::timeout::error::Elapsed`), return 408
/// - Otherwise, return 500
async fn handle_global_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long (timeout)".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {err}"),
        )
    }
}

pub fn create_app() -> Router {
    // 1) Start with a base Axum router that has your normal routes
    let base_router: Router = Router::new()
        .merge(hello_routes());

    // 2) Attach a single, combined ServiceBuilder layer that:
    //    a) Produces a timeout error after 5 seconds
    //    b) Converts that error into an HTTP response (making the service infallible again)
    //    c) Wraps all responses in your JSON "response_wrapper"
    //    d) Applies your "start_time_middleware"
    //
    // This single `.layer(...)` call means Axum still sees an Infallible error type at the end.
    base_router.layer(
        ServiceBuilder::new()
            // d) Track request durations (innermost layer)
            .layer(from_fn(start_time_middleware))

            // c) Wrap the final response with your universal JSON format
            .layer(from_fn(response_wrapper))

            // b) Convert that error into a valid HTTP response (408 or 500)
            .layer(HandleErrorLayer::new(handle_global_error))

            // a) If a request takes >5s, produce a tower::timeout error (outermost layer)
            .layer(TimeoutLayer::new(Duration::from_secs(5)))
    )
}
