use axum::{routing::post, Router};
use crate::config::state::AppState;
use super::handler;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(handler::register))
        .route("/auth/login", post(handler::login))
}

