// Database API route definitions

use axum::{
    routing::get,
    Router,
};

use crate::config::state::AppState;
use super::handler;

/// Creates router with database management endpoints
pub fn test_database_routes() -> Router<AppState> {
    Router::new()
        // General database monitoring
        .route("/db/health", get(handler::health_check))
        .route("/db/monitoring", get(handler::monitoring))
        // Tenant-specific monitoring for new single-schema architecture
        .route("/db/tenants/monitoring", get(handler::tenant_monitoring))
        .route("/db/schema/monitoring", get(handler::schema_monitoring))
}