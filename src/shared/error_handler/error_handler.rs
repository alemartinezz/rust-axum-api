// Start of file: /src/shared/error_handler/error_handler.rs

/*
    * Global error handler for middleware layers
*/

use axum::{
    http::StatusCode,
    response::IntoResponse,
    BoxError,
};
use std::error::Error;
use tower::timeout::error::Elapsed;
use http_body_util::LengthLimitError;

pub async fn handle_global_error(err: BoxError) -> impl IntoResponse {
    if find_cause::<LengthLimitError>(&*err).is_some() {
        return StatusCode::PAYLOAD_TOO_LARGE;
    }

    if err.is::<Elapsed>() {
        return StatusCode::REQUEST_TIMEOUT;
    }

    StatusCode::INTERNAL_SERVER_ERROR
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

// End of file: /src/shared/error_handler/error_handler.rs
