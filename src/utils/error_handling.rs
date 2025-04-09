// Start of file: src/utils/error_handling.rs

use axum::{
    http::StatusCode,
    response::IntoResponse,
    BoxError,
};
use http_body_util::LengthLimitError;
use std::error::Error;
use tower::timeout::error::Elapsed;

/*
    * Global error handler for middleware layers
*/
pub async fn handle_global_error(err: BoxError) -> impl IntoResponse {
    // Check for body length limit errors using our utility function.
    if let Some(e) = find_cause::<LengthLimitError>(&*err) {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("Request body too large: {}", e),
        );
    }

    // Check for request timeout errors.
    if let Some(e) = err.downcast_ref::<Elapsed>() {
        return (
            StatusCode::REQUEST_TIMEOUT,
            format!("Request timeout: {}", e),
        );
    }

    // Fallback to generic error.
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", err),
    )
}

/*
    * Helper function to find root cause of errors
*/
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

// End of file: src/utils/error_handling.rs
