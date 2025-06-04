// Database API endpoints demonstrating multi-tenant functionality

use serde_json::json;
use axum::{http::StatusCode, extract::State};

use crate::config::state::AppState;
use crate::utils::response_handler::HandlerResponse;
use tracing::{instrument, info, warn};

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

/// Get database monitoring information with global stats and instance grouping
#[instrument(skip(state))]
pub async fn monitoring(State(state): State<AppState>) -> HandlerResponse {
    info!("Database monitoring called");
    
    // Get local instance information
    let active_pools: Vec<String> = state.database.list_active_pools().await;
    let instance_id: String = get_instance_identifier();
    
    // Get global database statistics
    let global_stats: serde_json::Value = match get_global_db_stats(&state, &instance_id, &active_pools).await {
        Ok(stats) => stats,
        Err(e) => {
            warn!("Failed to get global database stats: {}", e);
            json!({
                "error": "Could not retrieve global stats",
                "details": e.to_string()
            })
        }
    };
    
    HandlerResponse::new(StatusCode::OK)
        .data(json!({
            "postgresql_cluster": global_stats
        }))
        .message("Database monitoring data retrieved")
}

/// Generate a unique identifier for this application instance
fn get_instance_identifier() -> String {
    // Try to get container ID first (for Docker environments)
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        if hostname.len() > 8 && hostname.chars().all(|c: char| c.is_ascii_hexdigit() || c == '-') {
            return format!("container_{}", &hostname[..12]); // Docker container ID
        }
        return format!("host_{}", hostname);
    }
    
    // Fallback to hostname
    if let Ok(hostname) = hostname::get() {
        if let Some(hostname_str) = hostname.to_str() {
            return format!("host_{}", hostname_str);
        }
    }
    
    // Last resort: generate a process-unique ID
    let process_id: u32 = std::process::id();
    format!("process_{}", process_id)
}

/// Get global database statistics from PostgreSQL
async fn get_global_db_stats(state: &AppState, instance_id: &str, active_pools: &[String]) -> anyhow::Result<serde_json::Value> {
    let master_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_master_pool().await?;
    
    // Get total active connections
    let total_active: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_stat_activity WHERE state = 'active'"
    )
    .fetch_one(&master_pool)
    .await?;
    
    // Get total connections (active + idle)
    let total_connections: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_stat_activity"
    )
    .fetch_one(&master_pool)
    .await?;
    
    // Get idle connections
    let idle_connections: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_stat_activity WHERE state = 'idle'"
    )
    .fetch_one(&master_pool)
    .await?;
    
    // Get connections by database
    let connections_by_db: Vec<(String, i64)> = sqlx::query_as(
        "SELECT datname, count(*) as conn_count 
         FROM pg_stat_activity 
         WHERE datname IS NOT NULL 
         GROUP BY datname 
         ORDER BY conn_count DESC"
    )
    .fetch_all(&master_pool)
    .await?;
    
    // Get max connections with proper error handling (server-wide setting)
    let max_connections_info: serde_json::Value = match get_max_connections(&master_pool).await {
        Ok(max_conn) => json!({
            "value": max_conn,
            "usage_percentage": if max_conn > 0 { 
                ((total_connections as f64 / max_conn as f64) * 100.0).round() as i64 
            } else { 0 },
            "status": "available"
        }),
        Err(e) => {
            warn!("Could not retrieve max_connections setting: {}", e);
            json!({
                "value": null,
                "usage_percentage": null,
                "status": "error",
                "error": "Unable to access PostgreSQL configuration"
            })
        }
    };
    
    // Build hierarchical structure: database -> applications within each database
    let mut databases_with_apps: Vec<serde_json::Value> = Vec::new();
    
    for (db_name, db_connections) in connections_by_db {
        // Get database size for this specific database
        let db_size: i64 = sqlx::query_scalar(
            "SELECT pg_database_size($1)"
        )
        .bind(&db_name)
        .fetch_one(&master_pool)
        .await.unwrap_or(0);
        
        // Convert to readable format
        let db_size_mb: i64 = db_size / (1024 * 1024);
        
        // Get applications within this specific database
        let apps_in_db: Vec<(String, i64)> = sqlx::query_as(
            "SELECT 
                COALESCE(application_name, 'unknown') as app_name, 
                count(*) as conn_count 
             FROM pg_stat_activity 
             WHERE datname = $1 
               AND application_name IS NOT NULL 
               AND application_name != ''
             GROUP BY application_name 
             ORDER BY conn_count DESC"
        )
        .bind(&db_name)
        .fetch_all(&master_pool)
        .await?;
        
        let connections_by_application: Vec<serde_json::Value> = apps_in_db.into_iter()
            .map(|(app, count)| {
                let mut app_data: serde_json::Value = json!({
                    "connections": count,
                    "percentage_of_database": if db_connections > 0 { 
                        ((count as f64 / db_connections as f64) * 100.0).round() as i64 
                    } else { 0 }
                });
                
                // If this is our rust-axum-api application, add instance metadata
                if app == "rust-axum-api" {
                    app_data.as_object_mut().unwrap().insert(
                        "instance_metadata".to_string(), 
                        json!({
                            "instance_id": instance_id,
                            "local_connection_pools": {
                                "active_pools": active_pools,
                                "pool_count": active_pools.len()
                            }
                        })
                    );
                }
                
                json!({
                    app: app_data
                })
            })
            .collect();
        
        // Database object - NO max_connections here, only size and applications
        databases_with_apps.push(json!({
            db_name: {
                "connections": db_connections,
                "percentage_of_server": if total_connections > 0 { 
                    ((db_connections as f64 / total_connections as f64) * 100.0).round() as i64 
                } else { 0 },
                "size": {
                    "size_mb": db_size_mb,
                    "size_gb": (db_size_mb as f64 / 1024.0 * 100.0).round() / 100.0
                },
                "applications": connections_by_application
            }
        }));
    }
    
    // Return with properly separated concerns: PostgreSQL server vs individual databases
    Ok(json!({
        "server": {
            "limits": {
                "max_connections": max_connections_info
            },
            "connections": {
                "total": total_connections,
                "breakdown": {
                    "active": total_active,
                    "idle": idle_connections,
                    "other": total_connections - total_active - idle_connections
                }
            }
        },
        "databases": databases_with_apps
    }))
}

/// Get PostgreSQL max_connections setting
async fn get_max_connections(pool: &sqlx::PgPool) -> anyhow::Result<i32> {
    let max_conn: i32 = sqlx::query_scalar(
        "SELECT setting::int FROM pg_settings WHERE name = 'max_connections'"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(max_conn)
} 