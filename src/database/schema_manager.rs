// =============================================================================
// DATABASE SERVICE - Multi-tenant PostgreSQL Management
// =============================================================================
// 
// Este módulo maneja toda la gestión de base de datos para el sistema multi-tenant:
// - Gestión de connection pools por schema
// - Creación y mantenimiento de schemas
// - Operaciones SQL basadas en archivos
// - Inicialización automática de tablas y funciones
//
// Estructura:
// 1. SQL Constants (cargados desde archivos)
// 2. DatabaseService struct y core functionality  
// 3. Schema Management methods
// 4. Connection Pool Management
// 5. Helper utilities
// =============================================================================

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool, Executor};
use tokio::sync::RwLock;
use tracing::{debug, info, log::LevelFilter, warn, error};

use crate::config::environment::EnvironmentVariables;

// =============================================================================
// SQL CONSTANTS - Loaded from files for better IDE support and maintainability
// =============================================================================

/// SQL para crear la función update_updated_at_column
/// Esta función es usada por triggers para actualizar automáticamente la columna updated_at
const CREATE_UPDATED_AT_FUNCTION: &str = include_str!("sql/schemas/functions.sql");

/// SQL para crear el esquema y tabla tenants
/// Reemplaza el enfoque de schema-per-tenant con Row-Level Security
const CREATE_TENANTS_SCHEMA: &str = include_str!("sql/schemas/tenants_schema.sql");

// =============================================================================
// DATABASE SERVICE - Main struct and core functionality
// =============================================================================

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

    /// Initializes the database service by setting up the tenants schema.
    /// Creates tenants schema with tenants table and required functions.
    /// Should be called once at application startup.
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing DatabaseService...");
        
        // First check if we can connect to the database at all
        self.verify_database_connectivity().await
            .context("Database connectivity check failed")?;
        
        let master_pool: sqlx::Pool<sqlx::Postgres> = self.get_master_pool().await?;
        self.initialize_database(&master_pool).await
            .context("Failed to initialize database")?;
        
        info!("DatabaseService initialized successfully");
        Ok(())
    }

    /// Verifies database connectivity with a quick connection test.
    /// This method uses a short timeout to fail fast when the database is not available.
    pub async fn verify_database_connectivity(&self) -> Result<()> {
        info!("Verifying database connectivity...");
        
        let connect_options: PgConnectOptions = self.create_connect_options(None).await?;
        
        // Use a very short timeout for the initial connection test
        let pool_result = tokio::time::timeout(
            std::time::Duration::from_secs(5), // 5 second timeout
            PgPoolOptions::new()
                .max_connections(1)
                .acquire_timeout(std::time::Duration::from_secs(3))
                .connect_with(connect_options)
        ).await;
        
        match pool_result {
            Ok(pool_result) => {
                match pool_result {
                    Ok(pool) => {
                        // Test the connection with a simple query
                        match sqlx::query("SELECT 1").fetch_one(&pool).await {
                            Ok(_) => {
                                info!("✅ Database connectivity verified successfully");
                                pool.close().await;
                                Ok(())
                            }
                            Err(e) => {
                                pool.close().await;
                                error!("❌ Database connection test query failed: {}", e);
                                Err(anyhow::anyhow!(
                                    "Database is reachable but query failed. Check credentials and database permissions: {}", e
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to connect to database: {}", e);
                        
                        // Provide specific error messages based on error type
                        let error_msg = if e.to_string().contains("Connection refused") || e.to_string().contains("No route to host") {
                            format!(
                                "PostgreSQL server is not running or not reachable at {}:{}.\n\
                                💡 Possible solutions:\n\
                                   • Start your PostgreSQL server or Docker container\n\
                                   • Check if the database host and port are correct\n\
                                   • Verify network connectivity\n\
                                Error details: {}", 
                                self.config.db_host, self.config.db_port, e
                            )
                        } else if e.to_string().contains("password authentication failed") {
                            format!(
                                "Authentication failed for database user '{}'.\n\
                                💡 Check your database credentials (DB_USER, DB_PASSWORD)\n\
                                Error details: {}", 
                                self.config.db_user, e
                            )
                        } else if e.to_string().contains("database") && e.to_string().contains("does not exist") {
                            format!(
                                "Database '{}' does not exist.\n\
                                💡 Create the database or check DB_NAME configuration\n\
                                Error details: {}", 
                                self.config.db_name, e
                            )
                        } else {
                            format!(
                                "Failed to connect to PostgreSQL database.\n\
                                💡 Check your database configuration:\n\
                                   • Host: {}\n\
                                   • Port: {}\n\
                                   • Database: {}\n\
                                   • User: {}\n\
                                Error details: {}", 
                                self.config.db_host, self.config.db_port, 
                                self.config.db_name, self.config.db_user, e
                            )
                        };
                        
                        Err(anyhow::anyhow!(error_msg))
                    }
                }
            }
            Err(_) => {
                error!(
                    "❌ Database connection timeout after 5 seconds. \
                    PostgreSQL server at {}:{} is not responding", 
                    self.config.db_host, self.config.db_port
                );
                Err(anyhow::anyhow!(
                    "Database connection timeout.\n\
                    💡 The PostgreSQL server is not responding. This usually means:\n\
                       • PostgreSQL/Docker is not running\n\
                       • Network connectivity issues\n\
                       • Firewall blocking the connection\n\
                    \n\
                    Please start your database server and try again."
                ))
            }
        }
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
}

// =============================================================================
// SCHEMA MANAGEMENT - SQL execution and schema operations
// =============================================================================

impl DatabaseService {
    /// Execute SQL from a file with proper error handling and logging
    async fn execute_sql(&self, pool: &PgPool, sql_content: &str, description: &str) -> Result<()> {
        info!("Executing schema operation: {}", description);
        
        match pool.execute(sql_content).await {
            Ok(_) => {
                info!("Successfully executed: {}", description);
                Ok(())
            }
            Err(e) => {
                error!("Failed to execute {}: {}", description, e);
                Err(anyhow::anyhow!("Schema operation failed: {}", e))
            }
        }
    }

    /// Create the update_updated_at_column function
    async fn create_update_function(&self, pool: &PgPool) -> Result<()> {
        self.execute_sql(pool, CREATE_UPDATED_AT_FUNCTION, "update_updated_at function creation").await
    }

    /// Create the tenants schema and table with triggers
    async fn create_tenants_schema(&self, pool: &PgPool) -> Result<()> {
        self.execute_sql(pool, CREATE_TENANTS_SCHEMA, "tenants schema and table creation").await
    }

    /// Initialize the database with tenants schema and master functions
    async fn initialize_database(&self, pool: &PgPool) -> Result<()> {
        // Ensure UTC timezone for this connection
        sqlx::query("SET timezone = 'UTC'")
            .execute(pool)
            .await
            .context("Failed to set timezone to UTC")?;

        // Create the update function first (required for triggers)
        self.create_update_function(pool).await?;
        
        // Create the tenants schema and table (includes triggers and RLS)
        // Uses CREATE SCHEMA/TABLE IF NOT EXISTS so existing data is preserved
        self.create_tenants_schema(pool).await?;
        
        info!("Database with tenants schema initialized successfully");
        Ok(())
    }

    /// Gets a pool configured to use the tenants schema
    pub async fn get_tenants_pool(&self) -> Result<PgPool> {
        self.get_pool("tenants", None).await
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
        // First ensure the update function exists
        self.create_update_function(pool).await?;

        let create_table_query = format!(
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

        // Create the trigger for automatic updated_at management
        let trigger_sql: String = format!(
            r#"
            DROP TRIGGER IF EXISTS update_{}_updated_at ON {};
            CREATE TRIGGER update_{}_updated_at
                BEFORE UPDATE ON {}
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column();
            "#,
            table_name, table_name, table_name, table_name
        );

        sqlx::query(&trigger_sql)
            .execute(pool)
            .await
            .context(format!("Failed to create trigger for table '{}'", table_name))?;

        info!("Created table '{}' with timestamps and triggers", table_name);
        Ok(())
    }

    /// Validate that a schema has all required tables
    pub async fn validate_schema(&self, pool: &PgPool, expected_tables: &[&str]) -> Result<bool> {
        for table_name in expected_tables {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_name = $1
                )"
            )
            .bind(table_name)
            .fetch_one(pool)
            .await?;

            if !exists {
                warn!("Table '{}' does not exist in schema", table_name);
                return Ok(false);
            }
        }
        
        info!("Schema validation passed for tables: {:?}", expected_tables);
        Ok(true)
    }
}

// =============================================================================
// CONNECTION POOL MANAGEMENT - Pool lifecycle and management
// =============================================================================

impl DatabaseService {
    /// Gets or creates a connection pool for the specified schema.
    /// Creates the schema if it doesn't exist and sets up required functions.
    /// For non-master schemas, also creates the update_updated_at_column function.
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

        // Ensure updated_at function exists in non-public schemas
        if schema_name != "master" && schema_name != "public" {
            self.create_update_function(&pool).await
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

    /// Gets the master schema pool (convenience method)
    pub async fn get_master_pool(&self) -> Result<PgPool> {
        self.get_pool("master", None).await
    }

    /// Gets a tenant schema pool (convenience method) - DEPRECATED
    /// Use get_tenants_pool() instead as we no longer use per-tenant schemas
    #[deprecated(note = "Use get_tenants_pool() instead")]
    pub async fn get_tenant_pool(&self, _tenant_id: &str, app: Option<&str>) -> Result<PgPool> {
        self.get_pool("tenants", app).await
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
}

// =============================================================================
// HELPER UTILITIES - Internal utilities and configuration
// =============================================================================

impl DatabaseService {
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

        // Configure search_path, timezone, and application_name
        if let Some(schema_name) = schema {
            options = options.options([
                ("search_path", schema_name), 
                ("timezone", "UTC"),
                ("application_name", "rust-axum-api")
            ]);
        } else {
            options = options.options([
                ("timezone", "UTC"),
                ("application_name", "rust-axum-api")
            ]);
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
}

impl Drop for DatabaseService {
    fn drop(&mut self) {
        // Note: async drop not available, graceful shutdown should call close_all_pools()
        debug!("DatabaseService dropped");
    }
} 