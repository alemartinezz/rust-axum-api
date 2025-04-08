use std::time::Duration;
use axum::{
    Router,
    http::StatusCode,
    error_handling::HandleErrorLayer,
    middleware::from_fn,
    BoxError,
    serve,
};
use tower::ServiceBuilder;
use tower::timeout::TimeoutLayer;
use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber;
use listenfd::ListenFd;

use my_axum_project::routes::hello_route;
use my_axum_project::middlewares::{start_time, response_wrapper};
use my_axum_project::models::state::AppState;

async fn handle_global_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Request took too long (timeout)".to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Unhandled internal error: {err}"))
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate: std::future::Pending<()> = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down gracefully");
        },
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // set up logging
    let env_filter: tracing_subscriber::EnvFilter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "my-axum-project=debug,tower_http=debug,axum=trace".parse().unwrap());
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .init();

    let state: AppState = AppState::from_env()?;

    // build our router
    let app: Router = Router::<AppState>::new()
        .merge(hello_route::hello_routes())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(start_time::start_time_middleware))
                .layer(from_fn(response_wrapper::response_wrapper))
                .layer(HandleErrorLayer::new(handle_global_error))
                .layer(TimeoutLayer::new(Duration::from_secs(5))),
        )
        .with_state(state.clone());

    // Listenfd integration
    let mut listenfd: ListenFd = ListenFd::from_env();
    let listener: TcpListener = match listenfd.take_tcp_listener(0)? {
        Some(std_listener) => {
            std_listener.set_nonblocking(true)?;
            TcpListener::from_std(std_listener)?
        }
        None => {
            let addr: String = format!("{}:{}", state.env.host, state.env.port);
            TcpListener::bind(&addr).await?
        }
    };

    println!("Server listening on: {}", listener.local_addr()?);
    
    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}