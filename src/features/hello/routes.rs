// Start of file: /src/features/hello/routes.rs

/*
    * This file defines the route(s) for the "hello" endpoint.
    * We register one GET route at `/hello` that calls `hello_handler`.
*/

use axum::{routing::get, Router};

use crate::features::hello::handler::hello_handler;
use crate::config::state::AppState;

pub fn hello_routes() -> Router<AppState> {
    Router::new().route("/hello", get(hello_handler))
}


// End of file: /src/features/hello/routes.rs
