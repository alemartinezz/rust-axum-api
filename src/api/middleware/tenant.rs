use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;
use crate::utils::response_handler::HandlerResponse;
use crate::config::state::AppState;
use serde_json::json;

/// Header key for Tenant ID
pub const TENANT_ID_HEADER: &str = "x-tenant-id";

/// Tenant Context to be stored in request extensions
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
}

/// Middleware to extract Tenant ID from header and set up context
pub async fn tenant_context_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, HandlerResponse> {
    // 1. Extract Tenant ID from Header
    let tenant_id_str: &str = headers
        .get(TENANT_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            HandlerResponse::new(StatusCode::UNAUTHORIZED)
                .message("Missing Tenant ID header")
                .data(json!({ "error": "missing_tenant_id" }))
        })?;

    // 2. Parse UUID
    let tenant_id: Uuid = Uuid::parse_str(tenant_id_str).map_err(|_| {
        HandlerResponse::new(StatusCode::BAD_REQUEST)
            .message("Invalid Tenant ID format")
            .data(json!({ "error": "invalid_tenant_id" }))
    })?;

    // 3. Validate against Cache/Redis (Optimization)
    // Check Redis first (Hot Path)
    let is_cached: bool = state.redis.tenant_exists(&tenant_id).await.unwrap_or(false);

    if !is_cached {
        // Not in cache, check Database
        let pool: &sqlx::PgPool = state.database.get_pool().map_err(|e| {
            tracing::error!("DB Pool error: {}", e);
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Internal Service Error")
        })?;

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM tenants WHERE id = $1)"
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("DB Query error: {}", e);
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Internal Service Error")
        })?;

        if !exists {
            return Err(HandlerResponse::new(StatusCode::UNAUTHORIZED)
                .message("Invalid Tenant ID")
                .data(json!({ "error": "tenant_not_found" })));
        }

        // Valid tenant, populate Redis
        if let Err(e) = state.redis.set_tenant(&tenant_id).await {
            // Don't fail request if cache fails, just log it
            tracing::warn!("Failed to cache tenant {}: {}", tenant_id, e);
        }
        
        tracing::debug!("Tenant {} validated via DB and cached", tenant_id);
    } else {
        tracing::debug!("Tenant {} validated via Redis cache", tenant_id);
    }

    // 4. Store in Request Extensions
    // This makes the tenant_id available to subsequent middleware and handlers
    request.extensions_mut().insert(TenantContext { tenant_id });

    // 5. Proceed
    Ok(next.run(request).await)
}
