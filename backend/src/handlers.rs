use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::auth::{create_jwt_token, CurrentUser};
use crate::crypto;
use crate::models::{ApiKey, ApiKeyResponse, CreateApiKeyRequest, CreateApiKeyResponse, PaginatedResponse, PaginationQuery, TokenPool, TokenPoolResponse, CreateTokenPoolRequest, UpdateTokenPoolRequest, UpdateApiKeyRequest, User};

pub type AppPool = SqlitePool;

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Welcome message", body = str)
    )
)]
pub async fn index() -> &'static str {
    "Welcome to TokenForest API"
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize, ToSchema)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize, ToSchema)]
pub struct StatsResponse {
    pub api_keys_count: i64,
    pub token_pools_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 409, description = "Username already exists", body = ErrorResponse)
    )
)]
pub async fn register(
    State(pool): State<AppPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    if payload.username.len() < 3 || payload.username.len() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username must be between 3 and 50 characters".to_string(),
            }),
        ));
    }

    if payload.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Password must be at least 6 characters".to_string(),
            }),
        ));
    }

    let password_hash = hash(&payload.password, DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to hash password".to_string(),
            }),
        )
    })?;

    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, datetime('now'))"
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
                .bind(&payload.username)
                .fetch_one(&pool)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Failed to fetch user".to_string(),
                        }),
                    )
                })?;

            let token = create_jwt_token(&user).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to create token".to_string(),
                    }),
                )
            })?;

            Ok(Json(AuthResponse {
                token,
                user: UserInfo {
                    id: user.id,
                    username: user.username,
                },
            }))
        }
        Err(sqlx::Error::Database(_)) => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Username already exists".to_string(),
            }),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create user".to_string(),
            }),
        )),
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    )
)]
pub async fn login(
    State(pool): State<AppPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
        })?;

    let user = user.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Invalid credentials".to_string(),
        }),
    ))?;

    let valid = verify(&payload.password, &user.password_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to verify password".to_string(),
            }),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        ));
    }

    let token = create_jwt_token(&user).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create token".to_string(),
            }),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
        },
    }))
}

#[utoipa::path(
    get,
    path = "/api/stats",
    responses(
        (status = 200, description = "User statistics", body = StatsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_stats(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
) -> Result<Json<StatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let api_keys_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE user_id = ?")
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

    let token_pools_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM token_pools WHERE user_id = ?")
        .bind(claims.sub)
        .fetch_one(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to count token pools".to_string(),
                }),
            )
        })?;

    Ok(Json(StatsResponse {
        api_keys_count,
        token_pools_count,
    }))
}

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
    axum::extract::Path(id): axum::extract::Path<i64>,
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
    axum::extract::Path(id): axum::extract::Path<i64>,
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
    axum::extract::Path(id): axum::extract::Path<i64>,
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

#[utoipa::path(
    get,
    path = "/api/token-pools",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Paginated list of token pools", body = PaginatedResponse<TokenPoolResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_token_pools(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<TokenPoolResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let page = pagination.page.max(1);
    let page_size = pagination.page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM token_pools WHERE user_id = ?")
        .bind(claims.sub)
        .fetch_one(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to count token pools".to_string(),
                }),
            )
        })?;

    let pools = sqlx::query_as::<_, TokenPool>(
        "SELECT * FROM token_pools WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
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
                error: "Failed to fetch token pools".to_string(),
            }),
        )
    })?;

    let items: Vec<TokenPoolResponse> = pools
        .into_iter()
        .map(|p| TokenPoolResponse {
            id: p.id,
            name: p.name,
            model_type: p.model_type,
            base_url: p.base_url,
            is_active: p.is_active != 0,
            created_at: p.created_at,
            updated_at: p.updated_at,
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
    path = "/api/token-pools",
    request_body = CreateTokenPoolRequest,
    responses(
        (status = 200, description = "Token pool created", body = TokenPoolResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_token_pool(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Json(payload): Json<CreateTokenPoolRequest>,
) -> Result<Json<TokenPoolResponse>, (StatusCode, Json<ErrorResponse>)> {
    if payload.name.is_empty() || payload.name.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Name must be between 1 and 100 characters".to_string(),
            }),
        ));
    }

    let api_key_encrypted = crypto::encrypt(&payload.api_key).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to encrypt API key: {}", e),
            }),
        )
    })?;

    let result = sqlx::query(
        "INSERT INTO token_pools (user_id, name, model_type, base_url, api_key_encrypted, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, datetime('now'), datetime('now'))"
    )
    .bind(claims.sub)
    .bind(&payload.name)
    .bind(&payload.model_type)
    .bind(&payload.base_url)
    .bind(&api_key_encrypted)
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create token pool".to_string(),
            }),
        )
    })?;

    let created = sqlx::query_as::<_, TokenPool>(
        "SELECT * FROM token_pools WHERE id = ?"
    )
    .bind(result.last_insert_rowid())
    .fetch_one(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch created token pool".to_string(),
            }),
        )
    })?;

    Ok(Json(TokenPoolResponse {
        id: created.id,
        name: created.name,
        model_type: created.model_type,
        base_url: created.base_url,
        is_active: created.is_active != 0,
        created_at: created.created_at,
        updated_at: created.updated_at,
    }))
}

#[utoipa::path(
    put,
    path = "/api/token-pools/{id}",
    request_body = UpdateTokenPoolRequest,
    responses(
        (status = 200, description = "Token pool updated", body = TokenPoolResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token pool not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_token_pool(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<UpdateTokenPoolRequest>,
) -> Result<Json<TokenPoolResponse>, (StatusCode, Json<ErrorResponse>)> {
    let existing = sqlx::query_as::<_, TokenPool>(
        "SELECT * FROM token_pools WHERE id = ? AND user_id = ?"
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch token pool".to_string(),
            }),
        )
    })?;

    let existing = existing.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Token pool not found".to_string(),
        }),
    ))?;

    let name = payload.name.unwrap_or(existing.name);
    let model_type = payload.model_type.unwrap_or(existing.model_type);
    let base_url = payload.base_url.unwrap_or(existing.base_url);
    let api_key_encrypted = match payload.api_key {
        Some(ref key) => crypto::encrypt(key).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to encrypt API key: {}", e),
                }),
            )
        })?,
        None => existing.api_key_encrypted,
    };

    sqlx::query(
        "UPDATE token_pools SET name = ?, model_type = ?, base_url = ?, api_key_encrypted = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(&name)
    .bind(&model_type)
    .bind(&base_url)
    .bind(&api_key_encrypted)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to update token pool".to_string(),
            }),
        )
    })?;

    let updated = sqlx::query_as::<_, TokenPool>(
        "SELECT * FROM token_pools WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch updated token pool".to_string(),
            }),
        )
    })?;

    Ok(Json(TokenPoolResponse::from(updated)))
}

#[utoipa::path(
    delete,
    path = "/api/token-pools/{id}",
    responses(
        (status = 200, description = "Token pool deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token pool not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_token_pool(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM token_pools WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(claims.sub)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete token pool".to_string(),
                }),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Token pool not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/token-pools/{id}/toggle",
    responses(
        (status = 200, description = "Token pool toggled", body = TokenPoolResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token pool not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn toggle_token_pool(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<TokenPoolResponse>, (StatusCode, Json<ErrorResponse>)> {
    let pool_item = sqlx::query_as::<_, TokenPool>(
        "SELECT * FROM token_pools WHERE id = ? AND user_id = ?"
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch token pool".to_string(),
            }),
        )
    })?;

    let pool_item = pool_item.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Token pool not found".to_string(),
        }),
    ))?;

    let new_active = if pool_item.is_active == 0 { 1 } else { 0 };

    sqlx::query("UPDATE token_pools SET is_active = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(new_active)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update token pool".to_string(),
                }),
            )
        })?;

    let updated = sqlx::query_as::<_, TokenPool>(
        "SELECT * FROM token_pools WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch updated token pool".to_string(),
            }),
        )
    })?;

    Ok(Json(TokenPoolResponse::from(updated)))
}

#[derive(Serialize, ToSchema)]
pub struct TestTokenPoolResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_content: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/token-pools/{id}/test",
    responses(
        (status = 200, description = "Test result", body = TestTokenPoolResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token pool not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn test_token_pool(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<TestTokenPoolResponse>, (StatusCode, Json<ErrorResponse>)> {
    let pool_item = sqlx::query_as::<_, TokenPool>(
        "SELECT * FROM token_pools WHERE id = ? AND user_id = ?"
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch token pool".to_string(),
            }),
        )
    })?;

    let pool_item = pool_item.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Token pool not found".to_string(),
        }),
    ))?;

    let api_key = crypto::decrypt(&pool_item.api_key_encrypted).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to decrypt API key: {}", e),
            }),
        )
    })?;

    let base_url = pool_item.base_url.trim_end_matches('/');
    let test_url = format!("{}/v1/chat/completions", base_url);

    let test_prompt = "你是谁 你支持什么国内功能";

    let chat_request = serde_json::json!({
        "model": pool_item.model_type,
        "messages": [
            {
                "role": "user",
                "content": test_prompt
            }
        ],
        "max_tokens": 500
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&test_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&chat_request)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
                let content = body["choices"][0]["message"]["content"]
                    .as_str()
                    .map(|s| s.to_string());

                Ok(Json(TestTokenPoolResponse {
                    success: true,
                    message: "API key is valid and connection successful".to_string(),
                    response_content: content,
                }))
            } else {
                let status = resp.status().as_u16();
                let error_body: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
                let error_msg = error_body["error"]["message"]
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("API returned status code: {}", status));
                Ok(Json(TestTokenPoolResponse {
                    success: false,
                    message: error_msg,
                    response_content: None,
                }))
            }
        }
        Err(e) => {
            let message = if e.is_timeout() {
                "Connection timed out".to_string()
            } else if e.is_connect() {
                format!("Failed to connect to {}: {}", base_url, e)
            } else {
                format!("Connection failed: {}", e)
            };
            Ok(Json(TestTokenPoolResponse {
                success: false,
                message,
                response_content: None,
            }))
        }
    }
}
