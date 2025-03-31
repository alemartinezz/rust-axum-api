// Start of file: /src/main.rs

use tokio::net::TcpListener;
use axum::{serve, Router};
use tracing_subscriber;
use my_axum_project::app;

#[tokio::main]
async fn main() {
    // 1) Initialize a default tracing subscriber that prints all INFO (and above) logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .init();

    
    // 2) Build the router from our `app.rs`
    let app: Router = app::create_app();

    // 3) Bind the app to a TCP listener on port 3000
    let listener: TcpListener = TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on http://{}", listener.local_addr().unwrap());

    // 4) Serve the app on the listener
    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

#[cfg(unix)]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let mut term_signal = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate(),
    )
    .expect("failed to install signal handler");

    tokio::select! {
        _ = ctrl_c => {},
        _ = term_signal.recv() => {},
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
}


// End of file: /src/main.rs
