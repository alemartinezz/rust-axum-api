// Start of file: src/main.tf

/*
    * The main entry point for the application. Initializes logging,
    * sets up the server with routes and middleware, and handles graceful shutdown.
*/

use std::time::Duration;
use std::error::Error;
use axum::{
    serve,
    Router,
    BoxError,
    http::StatusCode,
    extract::DefaultBodyLimit,
    response::IntoResponse,
    middleware::from_fn,
    error_handling::HandleErrorLayer,
};
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower::timeout::TimeoutLayer;
use tracing_subscriber::{EnvFilter, fmt};
use listenfd::ListenFd;
use http_body_util::LengthLimitError;

use my_axum_project::middlewares::{start_time, response_wrapper};
use my_axum_project::models::state::AppState;
use my_axum_project::routes::hello_route;

/*
    * The Tokio runtime is required for asynchronous I/O and concurrency.
*/
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber with an environment filter.
    let env_filter: EnvFilter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "my-axum-project=debug,tower_http=debug,axum=trace".parse().unwrap());
    
    fmt()
        .with_env_filter(env_filter)
        .with_span_events(fmt::format::FmtSpan::FULL)
        .init();

    // Build application state from environment variables.
    let state: AppState = AppState::from_env()?;

    // Construct our main application router with routes and layered middleware.
    let app: Router = Router::<AppState>::new()
        .merge(hello_route::hello_routes())
        .layer(
            ServiceBuilder::new()
                // Body-size limit to prevent excessive data from large requests.
                .layer(DefaultBodyLimit::max(state.env.max_request_body_size))
                
                // Our custom start_time middleware to track request durations.
                .layer(from_fn(start_time::start_time_middleware))
                
                // Our custom response_wrapper middleware to unify response format.
                .layer(from_fn(response_wrapper::response_wrapper))
                
                // Global error handling for timeouts, body-limit, etc.
                .layer(HandleErrorLayer::new(handle_global_error))
                
                // A 5-second timeout for each request.
                .layer(TimeoutLayer::new(Duration::from_secs(state.env.default_timeout_seconds))),
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

    // TODO: Auto reload not working
    // TODO: Make tests for current global errors

    println!("Server listening on: {}", listener.local_addr()?);
    
    // Launch axum's server with graceful shutdown.
    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn handle_global_error(err: BoxError) -> impl IntoResponse {
    // Check for body length limit errors using our utility function.
    if let Some(e) = find_cause::<LengthLimitError>(&*err) {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("Request body too large: {}", e),
        );
    }

    // Check for request timeout errors.
    if let Some(e) = err.downcast_ref::<tower::timeout::error::Elapsed>() {
        return (
            StatusCode::REQUEST_TIMEOUT,
            format!("Request timeout: {}", e),
        );
    }

    // Fallback to generic error.
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", err),
    )
}

/*
    * This helper function loops through error sources to find a specific cause.
*/
fn find_cause<T: Error + 'static>(err: &dyn Error) -> Option<&T> {
    let mut source: Option<&dyn Error> = err.source();
    
    while let Some(s) = source {
        if let Some(typed) = s.downcast_ref::<T>() {
            return Some(typed);
        }
        source = s.source();
    }

    None
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
