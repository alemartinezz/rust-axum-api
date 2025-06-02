// Start of file: /src/api/test/routes.rs

// Routes for test endpoints

use axum::{routing::{get, post}, Router};
use crate::api::test::handler::{hello_handler, timeout_test_handler, error_test_handler, body_size_test_handler, status_handler, not_found_test_handler};
use crate::config::state::AppState;

// Build a Router with all test endpoints
pub fn test_routes() -> Router<AppState> {
    Router::new()
        // Original hello endpoint
        .route("/hello", get(hello_handler))
        
        // Status endpoint - simple GET request
        .route("/test/status", get(status_handler))
        
        // Timeout test endpoint - testea el timeout default de 4 segundos
        .route("/test/timeout", get(timeout_test_handler))
        // Para permitir más tiempo en este endpoint específico, agregar:
        // .layer(TimeoutLayer::new(Duration::from_secs(60)))
        
        // Error test endpoint - GET request that returns 500
        .route("/test/error", get(error_test_handler))
        
        // Not found test endpoint - GET request that returns 404
        .route("/test/not-found", get(not_found_test_handler))
        
        // Body size test endpoint - testea el payload máximo default de 2MB
        .route("/test/body-size", post(body_size_test_handler))
        // Para permitir más payload en este endpoint específico, agregar:
        // .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
}

// End of file: /src/api/test/routes.rs