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
    
    match state.database.get_tenants_pool().await {
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
    let tenants_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_tenants_pool().await?;
    
    // Get total active connections
    let total_active: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_stat_activity WHERE state = 'active'"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    // Get total connections (active + idle)
    let total_connections: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_stat_activity"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    // Get idle connections
    let idle_connections: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_stat_activity WHERE state = 'idle'"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    // Get connections by database
    let connections_by_db: Vec<(String, i64)> = sqlx::query_as(
        "SELECT datname, count(*) as conn_count 
         FROM pg_stat_activity 
         WHERE datname IS NOT NULL 
         GROUP BY datname 
         ORDER BY conn_count DESC"
    )
    .fetch_all(&tenants_pool)
    .await?;
    
    // Get max connections with proper error handling (server-wide setting)
    let max_connections_info: serde_json::Value = match get_max_connections(&tenants_pool).await {
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
        .fetch_one(&tenants_pool)
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
        .fetch_all(&tenants_pool)
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

/// Get tenant-specific monitoring information for the new single-schema architecture
#[instrument(skip(state))]
pub async fn tenant_monitoring(State(state): State<AppState>) -> HandlerResponse {
    info!("Tenant monitoring called");
    
    match get_tenant_stats(&state).await {
        Ok(stats) => {
            HandlerResponse::new(StatusCode::OK)
                .data(json!({
                    "tenant_architecture": "single_schema_with_rls",
                    "schema": "tenants",
                    "statistics": stats
                }))
                .message("Tenant monitoring data retrieved")
        }
        Err(e) => {
            warn!("Failed to get tenant stats: {}", e);
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .data(json!({ 
                    "error": "Could not retrieve tenant statistics",
                    "details": e.to_string() 
                }))
                .message("Tenant monitoring failed")
        }
    }
}

/// Get comprehensive tenant statistics from the tenants.tenants table
async fn get_tenant_stats(state: &AppState) -> anyhow::Result<serde_json::Value> {
    let tenants_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_tenants_pool().await?;
    
    // Get total tenant count
    let total_tenants: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tenants.tenants"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    // Get active vs inactive tenants
    let active_tenants: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tenants.tenants WHERE is_active = true"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    let inactive_tenants: i64 = total_tenants - active_tenants;
    
    // Get tenant creation stats by date
    let recent_tenants: Vec<(String, i64)> = sqlx::query_as(
        "SELECT 
            DATE(created_at) as creation_date,
            COUNT(*) as count
         FROM tenants.tenants 
         WHERE created_at >= NOW() - INTERVAL '30 days'
         GROUP BY DATE(created_at)
         ORDER BY creation_date DESC
         LIMIT 30"
    )
    .fetch_all(&tenants_pool)
    .await?;
    
    // Get oldest and newest tenants
    let oldest_tenant: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT name, created_at FROM tenants.tenants ORDER BY created_at ASC LIMIT 1"
    )
    .fetch_optional(&tenants_pool)
    .await?;
    
    let newest_tenant: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT name, created_at FROM tenants.tenants ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(&tenants_pool)
    .await?;
    
    // Get table size information
    let table_size: Option<i64> = sqlx::query_scalar(
        "SELECT COALESCE(pg_total_relation_size('tenants.tenants'), 0)"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    let table_size = table_size.unwrap_or(0);
    
    let table_size_mb: f64 = table_size as f64 / (1024.0 * 1024.0);
    
    // Check Row-Level Security status
    let rls_enabled: bool = sqlx::query_scalar(
        "SELECT relrowsecurity FROM pg_class WHERE relname = 'tenants' AND relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'tenants')"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    // Get index information
    let indexes: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT 
            indexname,
            pg_relation_size(schemaname||'.'||indexname) as size_bytes,
            indexdef
         FROM pg_indexes 
         WHERE schemaname = 'tenants' AND tablename = 'tenants'"
    )
    .fetch_all(&tenants_pool)
    .await?;
    
    let indexes_info: Vec<serde_json::Value> = indexes.into_iter()
        .map(|(name, size, definition)| {
            json!({
                "name": name,
                "size_mb": (size as f64 / (1024.0 * 1024.0)).round() * 100.0 / 100.0,
                "definition": definition
            })
        })
        .collect();
    
    // Calculate growth metrics
    let growth_last_7_days: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tenants.tenants WHERE created_at >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    let growth_last_30_days: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tenants.tenants WHERE created_at >= NOW() - INTERVAL '30 days'"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    Ok(json!({
        "tenant_counts": {
            "total": total_tenants,
            "active": active_tenants,
            "inactive": inactive_tenants,
            "activity_rate_percentage": if total_tenants > 0 { 
                ((active_tenants as f64 / total_tenants as f64) * 100.0).round() as i64 
            } else { 0 }
        },
        "growth_metrics": {
            "new_tenants_last_7_days": growth_last_7_days,
            "new_tenants_last_30_days": growth_last_30_days,
            "daily_creation_history": recent_tenants.into_iter()
                .map(|(date, count)| json!({ "date": date, "count": count }))
                .collect::<Vec<_>>()
        },
        "tenant_timeline": {
            "oldest_tenant": oldest_tenant.map(|(name, created)| json!({
                "name": name,
                "created_at": created.to_rfc3339()
            })),
            "newest_tenant": newest_tenant.map(|(name, created)| json!({
                "name": name,
                "created_at": created.to_rfc3339()
            }))
        },
        "storage_metrics": {
            "table_size_mb": (table_size_mb * 100.0).round() / 100.0,
            "table_size_bytes": table_size,
            "indexes": indexes_info
        },
        "security": {
            "row_level_security_enabled": rls_enabled,
            "architecture": "single_schema_with_rls",
            "isolation_method": "row_level_security"
        }
    }))
}

/// Get schema-specific monitoring for the tenants schema
#[instrument(skip(state))]
pub async fn schema_monitoring(State(state): State<AppState>) -> HandlerResponse {
    info!("Schema monitoring called");
    
    match get_schema_stats(&state).await {
        Ok(stats) => {
            HandlerResponse::new(StatusCode::OK)
                .data(json!({
                    "schema": "tenants",
                    "statistics": stats
                }))
                .message("Schema monitoring data retrieved")
        }
        Err(e) => {
            warn!("Failed to get schema stats: {}", e);
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .data(json!({ 
                    "error": "Could not retrieve schema statistics",
                    "details": e.to_string() 
                }))
                .message("Schema monitoring failed")
        }
    }
}

/// Get detailed statistics about the tenants schema
async fn get_schema_stats(state: &AppState) -> anyhow::Result<serde_json::Value> {
    let tenants_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_tenants_pool().await?;
    
    // Get basic schema information
    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'tenants'"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    // Get table names
    let table_names: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'tenants'"
    )
    .fetch_all(&tenants_pool)
    .await?;
    
    // Get function count
    let function_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) 
         FROM pg_proc p 
         JOIN pg_namespace n ON p.pronamespace = n.oid 
         WHERE n.nspname IN ('tenants', 'public')"
    )
    .fetch_one(&tenants_pool)
    .await?;
    
    // Get tenant table size (simplified)
    let tenant_table_size: Option<i64> = sqlx::query_scalar(
        "SELECT pg_total_relation_size('tenants.tenants')"
    )
    .fetch_optional(&tenants_pool)
    .await?;
    
    let table_stats: Vec<serde_json::Value> = table_names.into_iter()
        .map(|name| {
            json!({
                "name": name,
                "status": "active"
            })
        })
        .collect();
    
    Ok(json!({
        "schema_name": "tenants",
        "table_count": table_count,
        "function_count": function_count,
        "total_size_mb": tenant_table_size.map(|s| (s as f64 / (1024.0 * 1024.0)).round() * 100.0 / 100.0).unwrap_or(0.0),
        "total_size_bytes": tenant_table_size.unwrap_or(0),
        "tables": table_stats,
        "architecture": {
            "design": "Single schema with Row-Level Security",
            "benefits": [
                "Unlimited tenant scaling",
                "Simplified backup and maintenance",
                "Efficient cross-tenant queries",
                "Consistent performance"
            ]
        }
    }))
} 