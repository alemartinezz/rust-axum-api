// Start of file: src/main.rs

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
use my_axum_project::routes::hello_route;
use my_axum_project::middlewares::start_time;
use my_axum_project::middlewares::response_wrapper;

async fn handle_global_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long (timeout)".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {err}"),
        )
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
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

    // Wait for either Ctrl+C or terminate signal, and log that we are shutting down.
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
async fn main() {
    // Create an environment filter. This will allow logs from your endpoint
    // as well as from external crates (like tower_http and axum) at the proper levels.
    let env_filter: tracing_subscriber::EnvFilter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            format!(
                "{}=debug,tower_http=debug,axum=trace",
                env!("CARGO_CRATE_NAME")
            )
            .parse()
            .unwrap()
        });

    // Combine the env filter with the formatting layer and enable full span events.
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .init();

    let app: Router = Router::new()
        .merge(hello_route::hello_routes())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(start_time::start_time_middleware))
                .layer(from_fn(response_wrapper::response_wrapper))
                .layer(HandleErrorLayer::new(handle_global_error))
                .layer(TimeoutLayer::new(Duration::from_secs(5))),
        );

    let listener: TcpListener = TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

// End of file: src/main.rs