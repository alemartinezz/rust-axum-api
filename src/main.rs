// Start of file: /src/main.rs

// * /src/main.rs
// * The main entry point. Initializes tracing, environment, sets up routes, and runs the server.

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
use my_axum_project::features::hello::routes::hello_routes;
use my_axum_project::shared::{
    error_handler::handle_global_error,
    response_handler::response_wrapper
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // * Initialize the tracing subscriber
    let env_filter: EnvFilter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "my_axum_project=info,tower_http=debug,axum=trace".parse().unwrap());
    
    fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::FULL)
        .init();

    // * Initialize environment variables
    let _ = EnvironmentVariables::instance();
    
    // * Create the application state
    let state: AppState = AppState::new();
    
    // * For convenience, get a reference to the environment
    let env: &EnvironmentVariables = EnvironmentVariables::instance();

    // * Build the router:
    // ? 1) Merges the /hello routes
    // ? 2) Attaches the response_wrapper middleware to unify JSON response
    // ? 3) Attaches a global error handler layer
    // ? 4) Sets a timeout layer
    // ? 5) Sets a max body-size limit
    let app: Router = Router::new()
        .merge(hello_routes())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(response_wrapper))
                .layer(HandleErrorLayer::new(handle_global_error))
                .layer(TimeoutLayer::new(Duration::from_secs(env.default_timeout_seconds)))
                .layer(DefaultBodyLimit::max(env.max_request_body_size))
        )
        .with_state(state);

    // * Try to reuse an existing socket (via ListenFd),
    // * or bind a new listener if none is provided
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

    // * Show listening address
    println!("Server listening on: {}", listener.local_addr()?);

    // * Start serving with graceful shutdown
    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    // * A future that completes on Ctrl+C
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Ctrl+C handler");
    };

    #[cfg(unix)]
    // * On Unix, also listen for a TERM signal
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Terminate signal handler")
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
