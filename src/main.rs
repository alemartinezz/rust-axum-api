// Start of file: src/main.tf

/*
    * The main entry point for the application. Initializes logging,
    * sets up the server with routes and middleware, and handles graceful shutdown.
*/

use std::time::Duration;
use listenfd::ListenFd;
use fmt::format::FmtSpan;
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

use my_axum_project::utils::error_handling::handle_global_error;
use my_axum_project::middlewares::response_wrapper;
use my_axum_project::models::state::AppState;
use my_axum_project::routes::hello_route;

// TODO: Make tests

/*
    * The Tokio runtime is required for asynchronous I/O and concurrency.
*/
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber with an environment filter.
    let env_filter: EnvFilter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "my_axum_project=info,tower_http=debug,axum=trace".parse().unwrap());
    
    fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::FULL)
        .init();

    // Build application state from environment variables.
    let state: AppState = AppState::from_env()?;

    // Construct our main application router with routes and layered middleware.
    let app: Router = Router::<AppState>::new()
        .merge(hello_route::hello_routes())
        .layer(
            ServiceBuilder::new()
                // Our custom response_wrapper middleware to unify response format.
                .layer(from_fn(response_wrapper::response_wrapper))

                // Global error handling for timeouts, body-limit, etc.
                .layer(HandleErrorLayer::new(handle_global_error))

                // A default timeout for each request.
                .layer(TimeoutLayer::new(Duration::from_secs(state.env.default_timeout_seconds)))
        
                // Body-size limit to prevent excessive data from large requests.
                .layer(DefaultBodyLimit::max(state.env.max_request_body_size))   
        )
        .with_state(state.clone());

    // Listenfd allows the server to receive an already-bound socket in certain environments.
    let mut listenfd: ListenFd = ListenFd::from_env();

    let listener: TcpListener = match listenfd.take_tcp_listener(0)? {
        Some(std_listener) => {
            std_listener.set_nonblocking(true)?;
            
            TcpListener::from_std(std_listener)?
        }
        None => {
            let addr: String = format!("{}:{}", state.env.host, state.env.port);
            
            // Bind to the specified host and port from the environment.
            TcpListener::bind(&addr).await?
        }
    };

    println!("Server listening on: {}", listener.local_addr()?);
    
    // Launch axum's server with graceful shutdown.
    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/*
    * Graceful shutdown triggered by Ctrl+C or TERM signal (Unix).
*/
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

// End of file: src/main.tf