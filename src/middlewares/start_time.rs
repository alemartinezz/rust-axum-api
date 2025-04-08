// Start of file: /src/middlewares/start_time.rs

/*
    * This middleware inserts the current `Instant` into the request extensions
    * so that the response_wrapper can measure how long each request took.
*/

use std::time::Instant;
use std::convert::Infallible;
use axum::{
    body::Body,
    http::Request,
    middleware::Next,
};

pub async fn start_time_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<axum::response::Response, Infallible> {
    let start: Instant = Instant::now();
    
    // * Insert the start Instant into the request's extensions for later retrieval.
    req.extensions_mut().insert(start);

    // * Pass the request down the chain.
    Ok(next.run(req).await)
}

// End of file: /src/middlewares/start_time.rs
