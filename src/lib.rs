// Library root for Rust Axum API with multi-tenant database support

pub mod api;
pub mod config;
pub mod core;
pub mod database;
pub mod utils;

pub use config::*;
pub use core::*;
pub use database::*;
pub use utils::*;

// Legacy aliases for backward compatibility
pub use crate::config::state as app_state;
pub use crate::core::server as app_server;
pub use crate::core::logging as app_logging;

pub use crate::database::DatabaseService;
pub use crate::config::environment::EnvironmentVariables;
pub use crate::config::state::AppState;
