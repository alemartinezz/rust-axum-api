// Start of file: /src/middlewares/start_time.rs

use std::time::Instant;
use std::convert::Infallible;
use axum::{
    body::Body,
    http::Request,
    middleware::Next,
};

pub async fn start_time_middleware(
    mut req: Request<Body>,
    next: Next<>,
) -> Result<axum::response::Response, Infallible> {
    let start: Instant = Instant::now();
    
    req.extensions_mut().insert(start);

    // Pass the request down the chain
    Ok(next.run(req).await)
}

// Start of file: /src/middlewares/start_time.rs