// Start of file: src/main.rs
mod config;
mod controllers;
mod middleware;
mod models;
mod routes;

use axum::Server;
use config::SERVER_ADDRESS;
use routes::register_routes;

#[tokio::main]
async fn main() {
    config::init();
    let app = register_routes();
    println!("Servidor escuchando en {}", SERVER_ADDRESS);
    Server::bind(&SERVER_ADDRESS.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}


// End of file: src/main.rs
