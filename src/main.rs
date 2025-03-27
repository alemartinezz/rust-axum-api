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
    // Inicializa la configuración (placeholder)
    config::init();

    // Construye la aplicación con las rutas y añade el middleware de formateo de respuesta.
    let app = register_routes();

    println!("Servidor escuchando en {}", SERVER_ADDRESS);
    Server::bind(&SERVER_ADDRESS.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
