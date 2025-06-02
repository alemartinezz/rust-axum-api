// Start of file: /src/main.rs

// The main entry point - simplified and focused on orchestration only

use axum::serve;

use my_axum_project::config::{state::AppState, environment::EnvironmentVariables};
use my_axum_project::core::{logging, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    logging::init_tracing();

    // Initialize environment variables
    let _ = EnvironmentVariables::instance();
    
    // Create the application state
    let state: AppState = AppState::new();
    
    // Setup the application router
    let app: axum::Router = server::create_app(state);
    
    // Setup the TCP listener
    let listener: tokio::net::TcpListener = server::setup_listener().await?;

    // Show listening address
    println!("Server listening on: {}", listener.local_addr()?);

    // Start serving with graceful shutdown
    // Errors are handled by the global error handler middleware
    serve(listener, app)
        .with_graceful_shutdown(server::shutdown_signal())
        .await?;

    Ok(())
}

// End of file: /src/main.rs
