// Start of file: /src/main.rs

use axum::{
    Router,
    routing::get,
    middleware::from_fn,
    serve
};
use tokio::net::TcpListener;

mod models;
mod routes;
mod middleware;

#[tokio::main]
async fn main() {
    // Build a router with your route(s)
    let app = Router::new()
        .route("/hello", get(routes::hello::hello_handler))
        // Apply the "wrap_in_response_format" middleware to ALL routes
        .layer(from_fn(middleware::wrap_in_response_format));

    // Bind a TcpListener
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind port 3000");
    println!("Listening on http://{}", listener.local_addr().unwrap());

    // Now serve using `axum::serve(...).await` (NO `.run()` in axum 0.8!)
    serve(listener, app).await.expect("server error");
}

// End of file: /src/main.rs
