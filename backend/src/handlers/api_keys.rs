use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use bcrypt::{hash, DEFAULT_COST};

use crate::auth::CurrentUser;
use crate::models::{ApiKey, ApiKeyResponse, CreateApiKeyRequest, CreateApiKeyResponse, PaginatedResponse, PaginationQuery, UpdateApiKeyRequest};
use super::{AppPool, ErrorResponse};

fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

#[utoipa::path(
    get,
    path = "/api/api-keys",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Paginated list of API keys", body = PaginatedResponse<ApiKeyResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_api_keys(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ApiKeyResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let page = pagination.page.max(1);
    let page_size = pagination.page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE user_id = ?")
        .bind(claims.sub)
        .fetch_one(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to count API keys".to_string(),
                }),
            )
        })?;

    let keys = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(claims.sub)
    .bind(page_size as i32)
    .bind(offset as i32)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch API keys".to_string(),
            }),
        )
    })?;

    let items: Vec<ApiKeyResponse> = keys
        .into_iter()
        .map(|k| ApiKeyResponse {
            id: k.id,
            name: k.name,
            model: k.model,
            prefix: k.prefix,
            is_active: k.is_active != 0,
            allowed_cidrs: k.allowed_cidrs.as_ref().and_then(|s| serde_json::from_str(s).ok()),
            last_used_at: k.last_used_at,
            created_at: k.created_at,
        })
        .collect();

    let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

    Ok(Json(PaginatedResponse {
        items,
        total,
        page,
        page_size,
        total_pages,
    }))
}

#[utoipa::path(
    post,
    path = "/api/api-keys",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 200, description = "API key created", body = CreateApiKeyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_api_key(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, (StatusCode, Json<ErrorResponse>)> {
    if payload.name.is_empty() || payload.name.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Name must be between 1 and 100 characters".to_string(),
            }),
        ));
    }

    let raw_key = generate_api_key();
    let prefix = format!("tf-{}", &raw_key[..8]);
    let key_hash = hash(&raw_key, DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to hash API key".to_string(),
            }),
        )
    })?;

    let allowed_cidrs_json = payload.allowed_cidrs.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());

    let result = sqlx::query(
        "INSERT INTO api_keys (user_id, name, model, key_hash, prefix, is_active, allowed_cidrs, created_at) VALUES (?, ?, ?, ?, ?, 1, ?, datetime('now'))"
    )
    .bind(claims.sub)
    .bind(&payload.name)
    .bind(&payload.model)
    .bind(&key_hash)
    .bind(&prefix)
    .bind(&allowed_cidrs_json)
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create API key".to_string(),
            }),
        )
    })?;

    Ok(Json(CreateApiKeyResponse {
        id: result.last_insert_rowid(),
        name: payload.name,
        model: payload.model,
        key: format!("tf-{}", raw_key),
        prefix,
        allowed_cidrs: payload.allowed_cidrs,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/api-keys/{id}",
    responses(
        (status = 200, description = "API key deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "API key not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_api_key(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM api_keys WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(claims.sub)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete API key".to_string(),
                }),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "API key not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/api-keys/{id}/toggle",
    responses(
        (status = 200, description = "API key toggled", body = ApiKeyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "API key not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn toggle_api_key(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiKeyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let key = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE id = ? AND user_id = ?"
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch API key".to_string(),
            }),
        )
    })?;

    let key = key.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "API key not found".to_string(),
        }),
    ))?;

    let new_active = if key.is_active == 0 { 1 } else { 0 };

    sqlx::query("UPDATE api_keys SET is_active = ? WHERE id = ?")
        .bind(new_active)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update API key".to_string(),
                }),
            )
        })?;

    let allowed_cidrs: Option<Vec<String>> = key.allowed_cidrs
        .and_then(|s| serde_json::from_str(&s).ok());

    Ok(Json(ApiKeyResponse {
        id: key.id,
        name: key.name,
        model: key.model,
        prefix: key.prefix,
        is_active: new_active != 0,
        allowed_cidrs,
        last_used_at: key.last_used_at,
        created_at: key.created_at,
    }))
}

#[utoipa::path(
    put,
    path = "/api/api-keys/{id}",
    request_body = UpdateApiKeyRequest,
    responses(
        (status = 200, description = "API key updated", body = ApiKeyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "API key not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_api_key(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> Result<Json<ApiKeyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let existing = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE id = ? AND user_id = ?"
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch API key".to_string(),
            }),
        )
    })?;

    let existing = existing.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "API key not found".to_string(),
        }),
    ))?;

    let name = payload.name.unwrap_or(existing.name);
    let model = payload.model.or(existing.model);
    let allowed_cidrs_json = match payload.allowed_cidrs {
        Some(ref v) => Some(serde_json::to_string(v).unwrap_or_default()),
        None => existing.allowed_cidrs,
    };

    sqlx::query(
        "UPDATE api_keys SET name = ?, model = ?, allowed_cidrs = ? WHERE id = ?"
    )
    .bind(&name)
    .bind(&model)
    .bind(&allowed_cidrs_json)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to update API key".to_string(),
            }),
        )
    })?;

    let allowed_cidrs: Option<Vec<String>> = allowed_cidrs_json
        .and_then(|s| serde_json::from_str(&s).ok());

    Ok(Json(ApiKeyResponse {
        id,
        name,
        model,
        prefix: existing.prefix,
        is_active: existing.is_active != 0,
        allowed_cidrs,
        last_used_at: existing.last_used_at,
        created_at: existing.created_at,
    }))
}
