// Tenant management handlers

use serde::{Deserialize, Serialize};
use serde_json::json;
use axum::{http::StatusCode, extract::State, Json};
use uuid::Uuid;
use sqlx::Row;

use crate::config::state::AppState;
use crate::utils::response_handler::HandlerResponse;
use tracing::{instrument, info, error};

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub tenant_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTenantResponse {
    pub id: Uuid,
    pub tenant_name: String,
    pub schema_name: String,
    pub created_at: String,
}

/// Creates a new tenant with its own schema and registers it in the master tenants table
#[instrument(name = "create_tenant", skip(state), fields(tenant_name = %request.tenant_name))]
pub async fn create_tenant_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateTenantRequest>,
) -> HandlerResponse {
    info!("Creating new tenant: {}", request.tenant_name);

    // Validate tenant name
    if request.tenant_name.trim().is_empty() {
        return HandlerResponse::new(StatusCode::BAD_REQUEST)
            .data(json!({ "error": "tenant_name cannot be empty" }))
            .message("Invalid tenant name provided");
    }

    if request.tenant_name.len() > 100 {
        return HandlerResponse::new(StatusCode::BAD_REQUEST)
            .data(json!({ "error": "tenant_name cannot exceed 100 characters" }))
            .message("Tenant name too long");
    }

    // Validate tenant name format (alphanumeric, underscores, hyphens only)
    if !request.tenant_name.chars().all(|c: char| c.is_alphanumeric() || c == '_' || c == '-') {
        return HandlerResponse::new(StatusCode::BAD_REQUEST)
            .data(json!({ "error": "tenant_name can only contain alphanumeric characters, underscores, and hyphens" }))
            .message("Invalid tenant name format");
    }

    let tenant_name: String = request.tenant_name.trim().to_lowercase();

    match create_tenant_internal(&state, &tenant_name).await {
        Ok(response) => {
            info!("Successfully created tenant: {} with schema: {}", response.tenant_name, response.schema_name);
            HandlerResponse::new(StatusCode::CREATED)
                .data(json!(response))
                .message("Tenant created successfully")
        }
        Err(e) => {
            error!("Failed to create tenant '{}': {}", tenant_name, e);
            
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .data(json!({ 
                    "error": "tenant_creation_failed",
                    "details": e.to_string() 
                }))
                .message("Failed to create tenant")
        }
    }
}

/// Internal function to create tenant schema and register in master table
async fn create_tenant_internal(
    state: &AppState,
    tenant_name: &str,
) -> anyhow::Result<CreateTenantResponse> {
    // Generate tenant ID
    let tenant_id: Uuid = Uuid::new_v4();
    let schema_name: String = format!("tenant_{}", tenant_id.simple());

    // Get master pool to register tenant
    let master_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_master_pool().await?;

    // Begin transaction for atomic operation
    let mut tx: sqlx::Transaction<'static, sqlx::Postgres> = master_pool.begin().await?;

    // Insert tenant record into master tenants table (only id and timestamps)
    let row: sqlx::postgres::PgRow = sqlx::query(
        r#"
        INSERT INTO tenants (id, created_at, updated_at)
        VALUES ($1, NOW(), NOW())
        RETURNING id, created_at
        "#,
    )
    .bind(tenant_id)
    .fetch_one(&mut *tx)
    .await?;

    let record_id: Uuid = row.get("id");
    let record_created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

    // Commit master transaction
    tx.commit().await?;

    // Create the tenant schema and its tables
    let tenant_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_pool(&schema_name, None).await?;
    
    // Create tenant_info table in the tenant schema
    state.database.create_tenant_info_table(&tenant_pool).await?;

    // Insert tenant name into tenant_info table in the tenant schema
    sqlx::query(
        r#"
        INSERT INTO tenant_info (name, created_at, updated_at)
        VALUES ($1, NOW(), NOW())
        "#,
    )
    .bind(tenant_name)
    .execute(&tenant_pool)
    .await?;

    let response: CreateTenantResponse = CreateTenantResponse {
        id: record_id,
        tenant_name: tenant_name.to_string(),
        schema_name,
        created_at: record_created_at.to_rfc3339(),
    };

    Ok(response)
}

/// Lists all existing tenants
#[instrument(name = "list_tenants", skip(state))]
pub async fn list_tenants_handler(
    State(state): State<AppState>,
) -> HandlerResponse {
    info!("Listing all tenants");

    match list_tenants_internal(&state).await {
        Ok(tenants) => {
            info!("Successfully retrieved {} tenants", tenants.len());
            HandlerResponse::new(StatusCode::OK)
                .data(json!({ "tenants": tenants, "count": tenants.len() }))
                .message("Tenants retrieved successfully")
        }
        Err(e) => {
            error!("Failed to list tenants: {}", e);
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .data(json!({ 
                    "error": "tenant_list_failed",
                    "details": e.to_string() 
                }))
                .message("Failed to retrieve tenants")
        }
    }
}

/// Internal function to list tenants from master table and their info from tenant schemas
/// OPTIMIZED: Reuses connections and implements better pool management
async fn list_tenants_internal(state: &AppState) -> anyhow::Result<Vec<serde_json::Value>> {
    let master_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_master_pool().await?;

    // Get all tenant IDs from master table
    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
        r#"
        SELECT id, created_at, updated_at
        FROM tenants
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&master_pool)
    .await?;

    let mut tenants: Vec<serde_json::Value> = Vec::new();

    // Para cada tenant, optimizar las conexiones
    for row in rows {
        let id: Uuid = row.get("id");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
        let schema_name = format!("tenant_{}", id.simple());

        // PROBLEMA INVESTIGADO: ¿Cada get_pool() está creando conexiones nuevas?
        // Vamos a examinar si el pool reuse está funcionando correctamente
        match state.database.get_pool(&schema_name, None).await {
            Ok(tenant_pool) => {
                match sqlx::query("SELECT name FROM tenant_info LIMIT 1")
                    .fetch_one(&tenant_pool)
                    .await
                {
                    Ok(tenant_info_row) => {
                        let tenant_name: String = tenant_info_row.get("name");
                        
                        tenants.push(json!({
                            "id": id,
                            "tenant_name": tenant_name,
                            "schema_name": schema_name,
                            "created_at": created_at.to_rfc3339(),
                            "updated_at": updated_at.to_rfc3339(),
                        }));
                    }
                    Err(e) => {
                        tracing::warn!("Could not fetch tenant name for {}: {}", schema_name, e);
                        tenants.push(json!({
                            "id": id,
                            "tenant_name": format!("tenant_{}", id.simple()),
                            "schema_name": schema_name,
                            "created_at": created_at.to_rfc3339(),
                            "updated_at": updated_at.to_rfc3339(),
                            "status": "schema_error"
                        }));
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Could not connect to tenant schema {}: {}", schema_name, e);
                tenants.push(json!({
                    "id": id,
                    "tenant_name": format!("tenant_{}", id.simple()),
                    "schema_name": schema_name,
                    "created_at": created_at.to_rfc3339(),
                    "updated_at": updated_at.to_rfc3339(),
                    "status": "connection_error"
                }));
            }
        }
    }

    Ok(tenants)
} 