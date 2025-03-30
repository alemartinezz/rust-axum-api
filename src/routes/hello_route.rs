// Start of file: /src/routes/hello.rs

use axum::{
    routing::get,
    Router,
};

use crate::controllers::hello_controller::hello_handler;

pub fn hello_routes() -> Router {
    Router::new()
        .route("/hello", get(hello_handler))
}

// End of file: /src/routes/hello.rs
