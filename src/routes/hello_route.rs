// Start of file: /src/routes/hello_route.rs

/*
    * This file defines the route(s) for the "hello" endpoint.
    * We register one GET route at `/hello` that calls `hello_handler`.
*/

use axum::{routing::get, Router};

use crate::controllers::hello_controller::hello_handler;
use crate::models::state::AppState;

pub fn hello_routes() -> Router<AppState> {
    // This sets up the GET /hello route with the hello_handler
    Router::new()
        .route("/hello", get(hello_handler))
}

// End of file: /src/routes/hello_route.rs
