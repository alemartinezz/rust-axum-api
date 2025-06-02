// Start of file: /src/config/state.rs

// Defines the `AppState` passed around to handlers and middleware.

use std::sync::Arc;
use crate::config::environment::EnvironmentVariables;

// AppState, wrapping our environment config in an Arc.
// You can add more fields here if you want to share other resources (e.g., DB pools).
#[derive(Debug, Clone)]
pub struct AppState {
    pub environment: Arc<&'static EnvironmentVariables>,
}

impl AppState {
    // Creates a new AppState, pulling from the environment singleton
    pub fn new() -> Self {
        Self {
            environment: Arc::new(EnvironmentVariables::instance()),
        }
    }
}

// End of file: /src/config/state.rs
