// Start of file: src/main.rs

mod config;
mod controllers;
mod middleware;
mod models;
mod routes;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use crate::routes::register_routes;

#[tokio::main]
async fn main() {
    // Optional config setup
    config::init();

    // Build the application router
    let app: axum::Router = register_routes();

    // Bind a TcpListener the same way official Axum examples do
    let addr: SocketAddr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
    let listener: TcpListener = TcpListener::bind(&addr).await.expect("failed to bind");

    println!("Servidor escuchando en {}", addr);

    // Run the server
    axum::serve(listener, app).await.unwrap();
}

// End of file: src/main.rs