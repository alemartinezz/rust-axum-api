// Database API endpoints demonstrating multi-tenant functionality

use serde_json::json;
use axum::{http::StatusCode, extract::State};

use crate::config::state::AppState;
use crate::utils::response_handler::HandlerResponse;
use tracing::{instrument, info};

/// Health check endpoint that verifies database connectivity
#[instrument(skip(state))]
pub async fn health_check(State(state): State<AppState>) -> HandlerResponse {
    info!("Database health check called");
    
    match state.database.get_master_pool().await {
        Ok(_) => {
            HandlerResponse::new(StatusCode::OK)
                .data(json!({ "database": "connected" }))
                .message("Database connection healthy")
        }
        Err(e) => {
            HandlerResponse::new(StatusCode::SERVICE_UNAVAILABLE)
                .data(json!({ "database": "disconnected", "error": e.to_string() }))
                .message("Database connection failed")
        }
    }
}

/// Get database monitoring information
#[instrument(skip(state))]
pub async fn monitoring(State(state): State<AppState>) -> HandlerResponse {
    info!("Database monitoring called");
    
    let active_pools: Vec<String> = state.database.list_active_pools().await;
    
    HandlerResponse::new(StatusCode::OK)
        .data(json!({
            "active_pools": active_pools,
            "pool_count": active_pools.len()
        }))
        .message("Database monitoring data retrieved")
} 