// Global error handling for HTTP middleware layers

use axum::{
    BoxError,
    http::StatusCode,
    response::IntoResponse,
    extract::rejection::MatchedPathRejection,
};
use std::error::Error;
// tower's error type for timeouts
use tower::timeout::error::Elapsed;
// Axum uses http_body_util for length-limiting
use http_body_util::LengthLimitError;

/// Maps various error types to appropriate HTTP responses
pub async fn handle_global_error(err: BoxError) -> impl IntoResponse {
    // 413 if the body was too large
    if find_cause::<LengthLimitError>(&*err).is_some() {
        return StatusCode::PAYLOAD_TOO_LARGE;
    }

    // 408 if the request took too long
    if err.is::<Elapsed>() {
        return StatusCode::REQUEST_TIMEOUT;
    }

    // 404 for not found routes/resources
    if find_cause::<MatchedPathRejection>(&*err).is_some() {
        return StatusCode::NOT_FOUND;
    }

    // Otherwise, 500
    StatusCode::INTERNAL_SERVER_ERROR
}

/// Helper function to find specific error type in error chain
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
