// Application server configuration and setup

use std::time::Duration;
use axum::{
    Router,
    middleware::from_fn,
    extract::DefaultBodyLimit,
    error_handling::HandleErrorLayer,
};
use tower::{ServiceBuilder, timeout::TimeoutLayer};
use tokio::{signal, net::TcpListener};
use listenfd::ListenFd;
use anyhow::Result;

use crate::config::state::AppState;
use crate::api::test_api::routes::test_api_routes;
use crate::api::test_database;
use crate::api::tenants::routes::tenant_routes;
use crate::utils::{
    error_handler::handle_global_error,
    response_handler::response_wrapper
};

/// Creates and configures the application router with all middleware layers
pub fn create_app() -> Router {
    let state: &'static AppState = AppState::instance();
    let env: &std::sync::Arc<crate::config::environment::EnvironmentVariables> = &state.environment;
    
    Router::new()
        .merge(test_api_routes())
        .merge(test_database::test_database_routes())
        // Add new routes here
        .merge(tenant_routes())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(response_wrapper))
                .layer(HandleErrorLayer::new(handle_global_error))
                .layer(TimeoutLayer::new(Duration::from_secs(env.default_timeout_seconds)))
                .layer(DefaultBodyLimit::max(env.max_request_body_size))
        )
        .with_state(state.clone())
}

/// Sets up the TCP listener from environment or binds to new address
pub async fn setup_listener() -> Result<TcpListener> {
    let env: &std::sync::Arc<crate::config::environment::EnvironmentVariables> = &AppState::instance().environment;
    let mut listenfd: ListenFd = ListenFd::from_env();
    
    let listener: TcpListener = match listenfd.take_tcp_listener(0)? {
        Some(std_listener) => {
            std_listener.set_nonblocking(true)?;
            TcpListener::from_std(std_listener)?
        }
        None => {
            let addr: String = format!("{}:{}", env.host, env.port);
            TcpListener::bind(&addr).await?
        }
    };
    
    Ok(listener)
}

/// Handles graceful shutdown signals (Ctrl+C and TERM)
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate: std::future::Pending<()> = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Shutting down via Ctrl+C"),
        _ = terminate => tracing::info!("Shutting down via TERM signal"),
    }
    
    // Gracefully close database connections
    AppState::shutdown().await;
}