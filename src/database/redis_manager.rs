use std::sync::Arc;
use anyhow::{Context, Result};
use redis::Client;
use tracing::info;
use crate::config::environment::EnvironmentVariables;

#[derive(Debug, Clone)]
pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub fn new(env: Arc<EnvironmentVariables>) -> Result<Self> {
        let client = Client::open(env.redis_url.as_ref())
            .context("Failed to create Redis client")?;
        Ok(Self { client })
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to connect to Redis")?;
        
        // Simple ping to verify connection
        let _: () = redis::cmd("PING").query_async(&mut conn).await
            .context("Failed to ping Redis")?;
            
        info!("Redis connection established successfully");
        Ok(())
    }

    pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis multiplexed connection")
    }

    pub async fn shutdown(&self) {
        // Redis client handles connection pooling/dropping automatically.
        // No explicit shutdown required for the client itself.
        info!("Redis service shutdown (noop)");
    }

    /// Checks if a tenant exists in the cache
    pub async fn tenant_exists(&self, tenant_id: &uuid::Uuid) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let key = format!("tenant:{}", tenant_id);
        
        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .context("Failed to check tenant existence in Redis")?;
            
        Ok(exists)
    }

    /// Caches a tenant as valid (with expiration)
    pub async fn set_tenant(&self, tenant_id: &uuid::Uuid) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = format!("tenant:{}", tenant_id);
        
        // Cache for 24 hours (or whatever policy)
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg("valid") // Value doesn't matter much, just existence
            .arg("EX")
            .arg(86400) // 24 hours
            .query_async(&mut conn)
            .await
            .context("Failed to cache tenant in Redis")?;
            
        Ok(())
    }
}
