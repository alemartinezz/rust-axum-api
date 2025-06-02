// Start of file: /src/core/logging.rs

use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::fmt::format::FmtSpan;

// Initialize the tracing subscriber with default configuration
pub fn init_tracing() {
    let env_filter: EnvFilter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "my_axum_project=info,tower_http=debug,axum=trace".parse().unwrap());
    
    fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::FULL)
        .init();
}

// End of file: /src/core/logging.rs