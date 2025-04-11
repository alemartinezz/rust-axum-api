// Start of file: /src/features/hello/routes.rs

// * Defines the /hello route, linking to our hello_handler.

use axum::{routing::get, Router};
use crate::features::hello::handler::hello_handler;
use crate::config::state::AppState;

// * Build a Router that has one route: GET /hello
pub fn hello_routes() -> Router<AppState> {
    Router::new().route("/hello", get(hello_handler))
}

// End of file: /src/features/hello/routes.rs
