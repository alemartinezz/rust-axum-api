//! tests/mod.rs
//! A shared test helper to spawn your Axum app on an ephemeral port.

use axum::error_handling::HandleErrorLayer;
use my_axum_project::config::{state::AppState, environment::EnvironmentVariables};
use my_axum_project::features::healthcheck::routes::healthcheck_routes;
use my_axum_project::shared::{
    error_handler::handle_global_error,
    response_handler::response_wrapper,
};

use axum::serve;
use std::time::Duration;
use tokio::net::TcpListener as TokioTcpListener;
use tower::{ServiceBuilder, timeout::TimeoutLayer};
use axum::{Router, extract::DefaultBodyLimit, middleware::from_fn};
use my_axum_project::features::tests::routes::tests_routes;
use tower_http::catch_panic::CatchPanicLayer;

/// Spawns the app on a random unused port and returns its base URL.
pub fn spawn_app() -> String {
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

    // * Bind an ephemeral port using std::net::TcpListener.
    let std_listener: std::net::TcpListener = 
        std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    
    std_listener.set_nonblocking(true).unwrap();

    // * Convert std::net::TcpListener to tokio::net::TcpListener.
    let tokio_listener: TokioTcpListener = TokioTcpListener::from_std(std_listener)
        .expect("Failed to convert to tokio listener");

    let addr: std::net::SocketAddr = tokio_listener.local_addr().unwrap();

    // * Spawn the server in a background task.
    tokio::spawn(async move {
        serve(tokio_listener, app)
            .await
            .expect("Server failed");
    });

    // * Return the base URL, e.g. "http://127.0.0.1:12345".
    format!("http://{}", addr)
}
