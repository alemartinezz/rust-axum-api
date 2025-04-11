// Start of file: /src/main.rs

use std::time::Duration;
use listenfd::ListenFd;
use tracing_subscriber::fmt::format::FmtSpan;
use axum::{
    serve,
    Router,
    middleware::from_fn,
    extract::DefaultBodyLimit,
    error_handling::HandleErrorLayer,
};
use tokio::{
    signal,
    net::TcpListener
};
use tower::{
    ServiceBuilder,
    timeout::TimeoutLayer
};
use tracing_subscriber::{
    fmt,
    EnvFilter
};

use my_axum_project::config::state::AppState;
use my_axum_project::config::environment::EnvironmentVariables;

// Import the two sets of routes
use my_axum_project::features::healthcheck::routes::healthcheck_routes;
use my_axum_project::features::tests::routes::tests_routes;
use tower_http::catch_panic::CatchPanicLayer;

use my_axum_project::shared::{
    error_handler::handle_global_error,
    response_handler::response_wrapper
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter: EnvFilter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "my_axum_project=info,tower_http=debug,axum=trace".parse().unwrap());
    
    fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::FULL)
        .init();

    // Load env config
    let _ = EnvironmentVariables::instance();
    let env: &EnvironmentVariables = EnvironmentVariables::instance();
    let state: AppState = AppState::new();

    let app: Router = Router::new()
        .merge(healthcheck_routes())
        .merge(tests_routes())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(response_wrapper))
                .layer(CatchPanicLayer::new())
                .layer(HandleErrorLayer::new(handle_global_error))
                .layer(TimeoutLayer::new(Duration::from_secs(env.default_timeout_seconds)))
                .layer(DefaultBodyLimit::max(env.max_request_body_size))
        )
        .with_state(state);

    // Bind or reuse a socket
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

    println!("Server listening on: {}", listener.local_addr()?);

    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("TERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Shutting down via Ctrl+C"),
        _ = terminate => tracing::info!("Shutting down via TERM signal"),
    }
}

// End of file: /src/main.rs
