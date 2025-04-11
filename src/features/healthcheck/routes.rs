// Start of file: /src/features/healthcheck/routes.rs

// * Defines the /healthcheck route, linking to our healthcheck_handler.

use axum::{routing::get, Router};
use crate::config::state::AppState;
use crate::features::healthcheck::handler::healthcheck_handler;

// * Build a Router that has one route: GET /healthcheck
pub fn healthcheck_routes() -> Router<AppState> {
    Router::new().route("/healthcheck", get(healthcheck_handler))
}

// End of file: /src/features/healthcheck/routes.rs
