// Application state management with singleton pattern

use std::sync::Arc;
use once_cell::sync::Lazy;
use crate::config::environment::EnvironmentVariables;
use crate::database::DatabaseService;

// AppState singleton
#[derive(Debug, Clone)]
pub struct AppState {
    pub environment: Arc<EnvironmentVariables>,
    pub database: DatabaseService,
}

impl AppState {
    /// Creates a new AppState instance (private constructor)
    fn new() -> anyhow::Result<Self> {
        let environment: EnvironmentVariables = EnvironmentVariables::load()?;
        let environment_arc: Arc<EnvironmentVariables> = Arc::new(environment);
        Ok(Self {
            environment: environment_arc.clone(),
            database: DatabaseService::new(environment_arc),
        })
    }

    /// Returns the singleton instance
    pub fn instance() -> &'static Self {
        static INSTANCE: Lazy<AppState> = Lazy::new(|| {
            AppState::new().expect("Failed to initialize AppState")
        });
        &INSTANCE
    }

    /// Initialize database with master schema and tenants table
    pub async fn init_master_schema() -> anyhow::Result<()> {
        let instance: &'static AppState = Self::instance();
        instance.database.initialize().await?;
        tracing::info!("Database initialized with master schema and tenants table");
        Ok(())
    }

    /// Gracefully shutdown all database connections
    pub async fn shutdown() {
        let instance: &'static AppState = Self::instance();
        instance.database.shutdown().await;
    }
}
