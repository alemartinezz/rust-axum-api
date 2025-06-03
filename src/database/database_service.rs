use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use tokio::sync::RwLock;
use tracing::{debug, info, log::LevelFilter, warn, error};

use crate::config::environment::EnvironmentVariables;

/// Multi-tenant database service that manages PostgreSQL connection pools per schema.
/// Each tenant gets its own isolated schema with dedicated connection pool.
/// Based on the TypeScript/NestJS DatabaseService pattern using SQLx.
#[derive(Clone, Debug)]
pub struct DatabaseService {
    /// Map of pool_key -> PgPool where key format is "{schema}_{app}" or just "{schema}"
    /// Uses RwLock for concurrent access optimized for reads
    data_sources: Arc<RwLock<HashMap<String, PgPool>>>,
    /// Environment configuration
    config: Arc<EnvironmentVariables>,
}

impl DatabaseService {
    /// Creates a new DatabaseService instance
    pub fn new(config: Arc<EnvironmentVariables>) -> Self {
        Self {
            data_sources: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Initializes the database service by setting up the master schema.
    /// Creates master schema with tenants table and required functions.
    /// Should be called once at application startup.
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing DatabaseService...");
        
        let master_pool: sqlx::Pool<sqlx::Postgres> = self.get_master_pool().await?;
        self.create_tenants_table(&master_pool).await
            .context("Failed to create tenants table in master schema")?;
        
        info!("DatabaseService initialized successfully");
        Ok(())
    }

    /// Creates the tenants table in the master schema with UTC timestamps
    async fn create_tenants_table(&self, pool: &PgPool) -> Result<()> {
        // Ensure UTC timezone for this connection
        sqlx::query("SET timezone = 'UTC'")
            .execute(pool)
            .await
            .context("Failed to set timezone to UTC")?;

        // Create the shared function for updated_at triggers
        self.create_updated_at_function(pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tenants (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_name VARCHAR NOT NULL UNIQUE,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create tenants table")?;
        
        self.create_updated_at_trigger(pool, "tenants").await?;
        
        Ok(())
    }

    /// Creates the update_updated_at_column function for automatic timestamp updates
    async fn create_updated_at_function(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $$ language 'plpgsql'
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create update_updated_at_column function")?;
        
        Ok(())
    }

    /// Creates an updated_at trigger for the specified table
    async fn create_updated_at_trigger(&self, pool: &PgPool, table_name: &str) -> Result<()> {
        let trigger_name: String = format!("update_{}_updated_at", table_name);
        
        // Drop existing trigger first
        let drop_query: String = format!(
            "DROP TRIGGER IF EXISTS {} ON {}",
            trigger_name, table_name
        );
        
        sqlx::query(&drop_query)
            .execute(pool)
            .await
            .context(format!("Failed to drop existing trigger for {}", table_name))?;
        
        // Create new trigger
        let create_query: String = format!(
            r#"
            CREATE TRIGGER {}
                BEFORE UPDATE ON {}
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column()
            "#,
            trigger_name, table_name
        );

        sqlx::query(&create_query)
            .execute(pool)
            .await
            .context(format!("Failed to create trigger for {} updated_at", table_name))?;
        
        Ok(())
    }

    /// Gets or creates a connection pool for the specified schema.
    /// Creates the schema if it doesn't exist and sets up required functions.
    /// For tenant schemas, also creates the update_updated_at_column function.
    pub async fn get_pool(&self, schema_name: &str, app: Option<&str>) -> Result<PgPool> {
        let pool_key: String = match app {
            Some(app_name) => format!("{}_{}", schema_name, app_name),
            None => schema_name.to_string(),
        };

        // Check for existing pool
        {
            let pools: tokio::sync::RwLockReadGuard<'_, HashMap<String, sqlx::Pool<sqlx::Postgres>>> = self.data_sources.read().await;
            if let Some(pool) = pools.get(&pool_key) {
                if !pool.is_closed() {
                    info!("Reusing existing pool for: {}", pool_key);
                    return Ok(pool.clone());
                } else {
                    warn!("Pool for {} exists but is closed. Will recreate.", pool_key);
                }
            }
        }

        info!("Creating new pool for: {}", pool_key);

        self.ensure_schema_exists(schema_name).await?;
        let pool: sqlx::Pool<sqlx::Postgres> = self.create_pool(schema_name).await?;

        // Ensure updated_at function exists in tenant schemas
        if schema_name != "master" {
            self.create_updated_at_function(&pool).await
                .context(format!("Failed to create updated_at function in schema '{}'", schema_name))?;
        }

        // Store the pool
        {
            let mut pools: tokio::sync::RwLockWriteGuard<'_, HashMap<String, sqlx::Pool<sqlx::Postgres>>> = self.data_sources.write().await;
            pools.insert(pool_key.clone(), pool.clone());
        }

        info!("Pool for '{}' initialized successfully", pool_key);
        Ok(pool)
    }

    /// Closes a specific connection pool
    pub async fn close_pool(&self, schema_name: &str, app: Option<&str>) -> Result<()> {
        let pool_key: String = match app {
            Some(app_name) => format!("{}_{}", schema_name, app_name),
            None => schema_name.to_string(),
        };

        let mut pools: tokio::sync::RwLockWriteGuard<'_, HashMap<String, sqlx::Pool<sqlx::Postgres>>> = self.data_sources.write().await;
        if let Some(pool) = pools.remove(&pool_key) {
            info!("Closing pool for schema '{}'...", pool_key);
            pool.close().await;
            info!("Pool for schema '{}' closed", pool_key);
        }
        Ok(())
    }

    /// Closes all connection pools
    pub async fn close_all_pools(&self) -> Result<()> {
        let mut pools: tokio::sync::RwLockWriteGuard<'_, HashMap<String, sqlx::Pool<sqlx::Postgres>>> = self.data_sources.write().await;
        for (key, pool) in pools.drain() {
            info!("Closing pool for schema '{}'...", key);
            pool.close().await;
            info!("Pool for schema '{}' closed", key);
        }
        Ok(())
    }

    /// Gracefully shuts down the service by closing all database connections.
    /// Logs errors but doesn't propagate them to avoid complicating shutdown logic.
    /// Designed to be called during application shutdown.
    pub async fn shutdown(&self) {
        info!("Initiating DatabaseService shutdown...");
        
        match self.close_all_pools().await {
            Ok(_) => info!("All database connections closed successfully"),
            Err(e) => error!("Error during database shutdown: {}", e),
        }
        
        info!("DatabaseService shutdown completed");
    }

    /// Ensures PostgreSQL schema exists, creating it if necessary.
    /// Skips creation for 'public' schema as it always exists.
    async fn ensure_schema_exists(&self, schema_name: &str) -> Result<()> {
        if schema_name == "public" {
            return Ok(());
        }

        info!("Ensuring schema '{}' exists...", schema_name);

        let connect_options: PgConnectOptions = self.create_connect_options(None).await?;
        let pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options)
            .await
            .context("Failed to connect to database for schema creation")?;

        let query: String = format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema_name);
        sqlx::query(&query)
            .execute(&pool)
            .await
            .context(format!("Failed to create schema '{}'", schema_name))?;

        pool.close().await;
        info!("Schema '{}' is ready", schema_name);
        Ok(())
    }

    /// Creates a new connection pool for the specified schema
    async fn create_pool(&self, schema: &str) -> Result<PgPool> {
        let connect_options: PgConnectOptions = self.create_connect_options(Some(schema)).await?;

        let pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .idle_timeout(std::time::Duration::from_secs(30))
            .connect_with(connect_options)
            .await
            .context(format!("Failed to create pool for schema '{}'", schema))?;

        Ok(pool)
    }

    /// Creates connection options with proper SSL and UTC timezone configuration
    async fn create_connect_options(&self, schema: Option<&str>) -> Result<PgConnectOptions> {
        let mut options: PgConnectOptions = PgConnectOptions::new()
            .host(&self.config.db_host)
            .port(self.config.db_port)
            .username(&self.config.db_user)
            .password(&self.config.db_password)
            .database(&self.config.db_name)
            .log_statements(LevelFilter::Debug);

        // Configure search_path and timezone
        if let Some(schema_name) = schema {
            options = options.options([("search_path", schema_name), ("timezone", "UTC")]);
        } else {
            options = options.options([("timezone", "UTC")]);
        }

        // Configure SSL based on environment
        let is_development: bool = self.config.environment == "development";
        if !is_development {
            // Production: require SSL
            options = options.ssl_mode(sqlx::postgres::PgSslMode::Require);
            
            // For custom SSL certificates:
            // let ssl_ca_path = Path::new("data/database/us-east-1-bundle.pem");
            // if ssl_ca_path.exists() {
            //     let ca_cert = fs::read(ssl_ca_path)?;
            //     options = options.ssl_root_cert_from_pem(ca_cert);
            // }
        } else {
            // Development: prefer SSL but don't require it
            options = options.ssl_mode(sqlx::postgres::PgSslMode::Prefer);
        }

        Ok(options)
    }

    /// Gets the master schema pool (convenience method)
    pub async fn get_master_pool(&self) -> Result<PgPool> {
        self.get_pool("master", None).await
    }

    /// Gets a tenant schema pool (convenience method)
    pub async fn get_tenant_pool(&self, tenant_id: &str, app: Option<&str>) -> Result<PgPool> {
        let schema_name: String = format!("tenant_{}", tenant_id);
        self.get_pool(&schema_name, app).await
    }

    /// Lists all active pool keys
    pub async fn list_active_pools(&self) -> Vec<String> {
        let pools: tokio::sync::RwLockReadGuard<'_, HashMap<String, sqlx::Pool<sqlx::Postgres>>> = self.data_sources.read().await;
        pools.keys().cloned().collect()
    }

    /// Gets connection pool statistics (total connections, idle connections)
    pub async fn get_pool_stats(&self, schema_name: &str, app: Option<&str>) -> Option<(usize, usize)> {
        let pool_key: String = match app {
            Some(app_name) => format!("{}_{}", schema_name, app_name),
            None => schema_name.to_string(),
        };

        let pools: tokio::sync::RwLockReadGuard<'_, HashMap<String, sqlx::Pool<sqlx::Postgres>>> = self.data_sources.read().await;
        if let Some(pool) = pools.get(&pool_key) {
            Some((pool.size() as usize, pool.num_idle()))
        } else {
            None
        }
    }

    /// Helper method to create tables with standard timestamp fields.
    /// Creates a table with id, created_at, updated_at fields and automatic trigger.
    /// Useful for maintaining consistent table structure across the application.
    pub async fn create_table_with_timestamps(
        &self,
        pool: &PgPool,
        table_name: &str,
        additional_columns: &str,
    ) -> Result<()> {
        let create_table_query: String = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                {},
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
            table_name, additional_columns
        );

        sqlx::query(&create_table_query)
            .execute(pool)
            .await
            .context(format!("Failed to create table '{}'", table_name))?;

        self.create_updated_at_trigger(pool, table_name).await?;

        Ok(())
    }
}

impl Drop for DatabaseService {
    fn drop(&mut self) {
        // Note: async drop not available, graceful shutdown should call close_all_pools()
        debug!("DatabaseService dropped");
    }
} 