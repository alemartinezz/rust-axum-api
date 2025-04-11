// Start of file: /src/shared/error_handler/error_handler.rs

// * Global error handling logic for layers (e.g. timeouts, large payloads).

use axum::{
    BoxError,
    http::StatusCode,
    response::IntoResponse,
};
use std::error::Error;
// * tower's error type for timeouts
use tower::timeout::error::Elapsed;
// * Axum uses http_body_util for length-limiting
use http_body_util::LengthLimitError;

use crate::shared::response_handler::HandlerResponse;

// ? This is the main function that maps errors to HTTP responses
pub async fn handle_global_error(err: BoxError) -> impl IntoResponse {
    // Decide the final status code
    let status: StatusCode = if find_cause::<LengthLimitError>(&*err).is_some() {
        StatusCode::PAYLOAD_TOO_LARGE
    } else if err.is::<Elapsed>() {
        StatusCode::REQUEST_TIMEOUT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    HandlerResponse::new(status)
}

// * A small helper function to find a specific cause in a chain of errors
pub fn find_cause<T: Error + 'static>(err: &dyn Error) -> Option<&T> {
    let mut source: Option<&dyn Error> = err.source();
    
    while let Some(s) = source {
        if let Some(typed) = s.downcast_ref::<T>() {
            return Some(typed);
        }
        source = s.source();
    }

    None
}

// End of file: /src/shared/error_handler/error_handler.rs
