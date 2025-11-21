use axum::{extract::{State, Extension}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use redis::AsyncCommands; // For Redis async operations

use crate::config::state::AppState;
use crate::utils::response_handler::HandlerResponse;
use crate::api::middleware::tenant::TenantContext;

// =============================================================================
// DTOs
// =============================================================================

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
}

// =============================================================================
// HANDLERS
// =============================================================================

/// Register a new user for the current tenant
pub async fn register(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
    Json(payload): Json<RegisterRequest>,
) -> HandlerResponse {
    // 1. Hash Password
    let password_hash: String = match hash(payload.password.as_bytes(), DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            return HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to process password")
                .data(json!({ "error": e.to_string() }));
        }
    };

    // 2. Insert User (Scoped Execution)
    // We use with_tenant to ensure the query runs with "SET LOCAL app.current_tenant_id = ..."
    // We must cast the transaction to &mut sqlx::PgConnection or Executor
    let result: anyhow::Result<sqlx::postgres::PgRow> = state.database.with_tenant(ctx.tenant_id, |tx| Box::pin(async move {
        // We need to reborrow tx as mutable for sqlx
        sqlx::query(
            r#"
            INSERT INTO users (tenant_id, email, password_hash, full_name)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#
        )
        .bind(ctx.tenant_id)
        .bind(payload.email)
        .bind(password_hash)
        .bind(payload.full_name)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| e.into()) // Convert sqlx::Error to anyhow::Error
    })).await;

    match result {
        Ok(row) => {
            use sqlx::Row;
            let user_id: Uuid = row.get("id");
            
            HandlerResponse::new(StatusCode::CREATED)
                .message("User registered successfully")
                .data(json!({ "user_id": user_id }))
        }
        Err(e) => {
            // Handle duplicate email error (Postgres error code 23505)
            if let Some(db_error) = e.downcast_ref::<sqlx::Error>() {
                 if let sqlx::Error::Database(db_err) = db_error {
                     if db_err.code().as_deref() == Some("23505") {
                        return HandlerResponse::new(StatusCode::CONFLICT)
                            .message("Email already registered")
                            .data(json!({ "error": "duplicate_email" }));
                     }
                 }
            }

            tracing::error!("Registration failed: {}", e);
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Registration failed")
                .data(json!({ "error": e.to_string() }))
        }
    }
}

/// Login and create a session
pub async fn login(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
    Json(payload): Json<LoginRequest>,
) -> HandlerResponse {
    // 1. Fetch User (Scoped Execution)
    // Clone email for the query closure
    let email_for_query: String = payload.email.clone();
    let user_result: anyhow::Result<Option<sqlx::postgres::PgRow>> = state.database.with_tenant(ctx.tenant_id, |tx| Box::pin(async move {
        sqlx::query(
            r#"
            SELECT id, password_hash
            FROM users
            WHERE email = $1
            "#
        )
        .bind(email_for_query) // RLS handles tenant_id filtering automatically? 
                             // IMPORTANT: We manually bind tenant_id in INSERT, but for SELECT 
                             // the RLS policy `tenant_id = current_setting(...)` handles it.
                             // However, the users table has a composite unique key (tenant_id, email).
                             // So multiple tenants can have the same email.
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| e.into()) // Convert sqlx::Error to anyhow::Error
    })).await;

    match user_result {
        Ok(Some(row)) => {
            use sqlx::Row;
            let user_id: Uuid = row.get("id");
            let stored_hash: String = row.get("password_hash");

            // 2. Verify Password
            if verify(payload.password.as_bytes(), &stored_hash).unwrap_or(false) {
                // 3. Generate Session Token
                let session_token: String = Uuid::new_v4().to_string();
                let redis_key: String = format!("session:{}", session_token);

                // 4. Store Session in Redis (TTL 24h)
                let mut conn: redis::aio::MultiplexedConnection = match state.redis.get_connection().await {
                    Ok(c) => c,
                    Err(e) => {
                        return HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                            .message("Failed to connect to cache")
                            .data(json!({ "error": e.to_string() }));
                    }
                };

                // Store user info in session
                let session_data: String = json!({
                    "user_id": user_id,
                    "tenant_id": ctx.tenant_id,
                    "email": payload.email
                }).to_string();

                let set_result: redis::RedisResult<()> = conn.set_ex(&redis_key, session_data, 24 * 60 * 60).await;
                
                match set_result {
                    Ok(_) => (),
                    Err(e) => {
                         return HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                            .message("Failed to create session")
                            .data(json!({ "error": e.to_string() }));
                    }
                };

                // 5. Return Token
                HandlerResponse::new(StatusCode::OK)
                    .message("Login successful")
                    .data(json!(AuthResponse {
                        token: session_token,
                        user_id,
                    }))

            } else {
                HandlerResponse::new(StatusCode::UNAUTHORIZED)
                    .message("Invalid credentials")
            }
        }
        Ok(None) => {
            HandlerResponse::new(StatusCode::UNAUTHORIZED)
                .message("Invalid credentials")
        }
        Err(e) => {
            tracing::error!("Login failed: {}", e);
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Login failed")
                .data(json!({ "error": e.to_string() }))
        }
    }
}

