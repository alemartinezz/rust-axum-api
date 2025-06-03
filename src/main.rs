// Main application entry point

use axum::serve;

use my_axum_project::config::state::AppState;
use my_axum_project::core::{logging, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::init_tracing();
    
    // Initialize database with master schema and tenants table
    AppState::init_master_schema().await?;
    
    let app: axum::Router = server::create_app();
    let listener: tokio::net::TcpListener = server::setup_listener().await?;

    println!("Server listening on: {}", listener.local_addr()?);

    // Start server with graceful shutdown handling
    serve(listener, app)
        .with_graceful_shutdown(server::shutdown_signal())
        .await?;

    Ok(())
}
