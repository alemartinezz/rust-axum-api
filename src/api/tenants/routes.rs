// Tenant management route definitions

use axum::{
    routing::{get, post},
    Router,
};

use crate::config::state::AppState;
use super::handler;

/// Creates router with all tenant management endpoints
pub fn tenant_routes() -> Router<AppState> {
    Router::new()
        .route("/tenants", post(handler::create_tenant_handler))
        .route("/tenants", get(handler::list_tenants_handler))
} 