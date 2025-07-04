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
    pub is_active: bool,
    pub created_at: String,
}

/// Creates a new tenant and registers it in the tenants.tenants table using Row-Level Security
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
            info!("Successfully created tenant: {}", response.tenant_name);
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

/// Internal function to create tenant in tenants schema
async fn create_tenant_internal(
    state: &AppState,
    tenant_name: &str,
) -> anyhow::Result<CreateTenantResponse> {
    // Get tenants pool to register tenant
    let tenants_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_tenants_pool().await?;

    // Insert tenant record into tenants.tenants table
    let row: sqlx::postgres::PgRow = sqlx::query(
        r#"
        INSERT INTO tenants.tenants (name, is_active, created_at, updated_at)
        VALUES ($1, $2, NOW(), NOW())
        RETURNING id, name, is_active, created_at
        "#,
    )
    .bind(tenant_name)
    .bind(true)    // Default active status
    .fetch_one(&tenants_pool)
    .await?;

    let record_id: Uuid = row.get("id");
    let record_name: String = row.get("name");
    let record_is_active: bool = row.get("is_active");
    let record_created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

    let response: CreateTenantResponse = CreateTenantResponse {
        id: record_id,
        tenant_name: record_name,
        is_active: record_is_active,
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

/// Internal function to list tenants from tenants.tenants table
/// SIMPLIFIED: Single query to tenants schema, no need for multiple connections
async fn list_tenants_internal(state: &AppState) -> anyhow::Result<Vec<serde_json::Value>> {
    let tenants_pool: sqlx::Pool<sqlx::Postgres> = state.database.get_tenants_pool().await?;

    // Get all tenants from tenants.tenants table
    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
        r#"
        SELECT id, name, is_active, created_at, updated_at
        FROM tenants.tenants
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&tenants_pool)
    .await?;

    let mut tenants: Vec<serde_json::Value> = Vec::new();

    // Build tenant list from single table
    for row in rows {
        let id: Uuid = row.get("id");
        let name: String = row.get("name");
        let is_active: bool = row.get("is_active");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        tenants.push(json!({
            "id": id,
            "tenant_name": name,
            "is_active": is_active,
            "created_at": created_at.to_rfc3339(),
            "updated_at": updated_at.to_rfc3339(),
        }));
    }

    Ok(tenants)
} 