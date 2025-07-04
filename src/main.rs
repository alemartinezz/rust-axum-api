// Main application entry point

use axum::serve;
use tracing::{error, info, warn};

use my_axum_project::config::state::AppState;
use my_axum_project::core::{logging, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging first
    logging::init_tracing();
    
    info!("🚀 Starting IC360 API server...");
    
    // Initialize database with proper error handling
    match AppState::init_master_schema().await {
        Ok(_) => {
            info!("✅ Database initialization completed successfully");
        }
        Err(e) => {
            error!("💥 Failed to initialize database connection");
            eprintln!("\n🔴 DATABASE CONNECTION FAILED");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("{}", e);
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("\n🛠️  Quick Fix:");
            eprintln!("   1. Start PostgreSQL with Docker:");
            eprintln!("      cd ic360-api && docker-compose -f docker/docker-compose.dev.yml up -d");
            eprintln!("");
            eprintln!("   2. Or start local PostgreSQL:");
            eprintln!("      sudo systemctl start postgresql");
            eprintln!("");
            eprintln!("   3. Create .env file if missing:");
            eprintln!("      Copy configuration values from README.md or docker-compose.dev.yml");
            eprintln!("");
            
            // Perform graceful shutdown
            warn!("Initiating graceful shutdown due to database connection failure...");
            AppState::shutdown().await;
            
            // Exit with error code
            std::process::exit(1);
        }
    }
    
    // Create application and setup listener
    let app: axum::Router = server::create_app();
    let listener: tokio::net::TcpListener = match server::setup_listener().await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to setup server listener: {}", e);
            AppState::shutdown().await;
            return Err(e);
        }
    };

    let server_addr = listener.local_addr()?;
    info!("🌐 Server listening on: {}", server_addr);
    info!("📡 API endpoints available at: http://{}", server_addr);
    info!("🏥 Health check: http://{}/health", server_addr);
    
    // Start server with graceful shutdown handling
    if let Err(e) = serve(listener, app)
        .with_graceful_shutdown(server::shutdown_signal())
        .await
    {
        error!("Server error: {}", e);
        AppState::shutdown().await;
        return Err(e.into());
    }

    info!("👋 Server shutdown completed");
    Ok(())
}
