// Application state management with singleton pattern

use std::sync::Arc;
use once_cell::sync::Lazy;
use crate::config::environment::EnvironmentVariables;
use crate::database::{DatabaseService, RedisService};

// AppState singleton
#[derive(Debug, Clone)]
pub struct AppState {
    pub environment: Arc<EnvironmentVariables>,
    pub database: DatabaseService,
    pub redis: RedisService,
}

impl AppState {
    /// Creates a new AppState instance (private constructor)
    fn new() -> anyhow::Result<Self> {
        let environment: EnvironmentVariables = EnvironmentVariables::load()?;
        let environment_arc: Arc<EnvironmentVariables> = Arc::new(environment);
        
        // Create services
        let database: DatabaseService = DatabaseService::new(environment_arc.clone());
        let redis: RedisService = RedisService::new(environment_arc.clone())?;

        Ok(Self {
            environment: environment_arc,
            database,
            redis,
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
        
        // Initialize both DB and Redis
        instance.database.initialize().await?;
        instance.redis.initialize().await?;
        
        tracing::info!("Services (DB + Redis) initialized successfully");
        Ok(())
    }

    /// Gracefully shutdown all database connections
    pub async fn shutdown() {
        let instance: &'static AppState = Self::instance();
        instance.database.shutdown().await;
        instance.redis.shutdown().await;
    }
}
