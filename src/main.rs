// Start of file: /src/main.rs

use tokio::net::TcpListener;
use axum::{serve, Router};

// Pull in everything else
mod app;
mod controllers;
mod middlewares;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    // 1) Initialize a default tracing subscriber that prints all INFO (and above) logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // 1) Build the router from our `app.rs`
    let app: Router = app::create_app();

    // 2) Bind and serve
    let listener: TcpListener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind port 3000");

    println!("Listening on http://{}", listener.local_addr().unwrap());

    serve(listener, app)
        .await
        .expect("server error");
}

// End of file: /src/main.rs
