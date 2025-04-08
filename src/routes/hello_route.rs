// Start of file: /src/routes/hello_route.rs

use axum::{routing::get, Router};
use crate::controllers::hello_controller::hello_handler;
use crate::models::state::AppState;

pub fn hello_routes() -> Router<AppState> {
    Router::new()
        .route("/hello", get(hello_handler))
}

// End of file: /src/routes/hello_route.rs
