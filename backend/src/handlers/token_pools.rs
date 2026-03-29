use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::CurrentUser;
use crate::crypto;
use crate::models::{
    CreateTokenPoolRequest, PaginatedResponse, PaginationQuery, TokenPool, TokenPoolResponse,
    UpdateTokenPoolRequest,
};

use super::{AppPool, ErrorResponse};

#[derive(Serialize, ToSchema)]
pub struct TestTokenPoolResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_content: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ChatResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
    get,
    path = "/api/token-pools/{id}",
    responses(
        (status = 200, description = "Token pool details", body = TokenPoolResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token pool not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_token_pool(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Path(id): Path<i64>,
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

    Ok(Json(TokenPoolResponse::from(pool_item)))
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
    Path(id): Path<i64>,
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
    Path(id): Path<i64>,
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
    Path(id): Path<i64>,
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
    Path(id): Path<i64>,
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

    let test_prompt = "你是谁 你支持什么功能";

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

#[utoipa::path(
    post,
    path = "/api/token-pools/{id}/chat",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Chat response", body = ChatResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token pool not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn chat_with_token_pool(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
    Path(id): Path<i64>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
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
    let chat_url = format!("{}/v1/chat/completions", base_url);

    let chat_request = serde_json::json!({
        "model": pool_item.model_type,
        "messages": request.messages,
        "max_tokens": request.max_tokens.unwrap_or(1000)
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&chat_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&chat_request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
                let content = body["choices"][0]["message"]["content"]
                    .as_str()
                    .map(|s| s.to_string());

                Ok(Json(ChatResponse {
                    success: true,
                    content,
                    error: None,
                }))
            } else {
                let status = resp.status().as_u16();
                let error_body: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
                let error_msg = error_body["error"]["message"]
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("API returned status code: {}", status));
                Ok(Json(ChatResponse {
                    success: false,
                    content: None,
                    error: Some(error_msg),
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
            Ok(Json(ChatResponse {
                success: false,
                content: None,
                error: Some(message),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"gpt-4\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_chat_response_success_serialization() {
        let response = ChatResponse {
            success: true,
            content: Some("Hello, world!".to_string()),
            error: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"content\":\"Hello, world!\""));
    }

    #[test]
    fn test_chat_response_error_serialization() {
        let response = ChatResponse {
            success: false,
            content: None,
            error: Some("Connection failed".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"Connection failed\""));
    }

    #[test]
    fn test_chat_message_serialization() {
        let message = ChatMessage {
            role: "assistant".to_string(),
            content: "Response text".to_string(),
        };
        let json = serde_json::to_string(&message).unwrap();
        assert_eq!(json, r#"{"role":"assistant","content":"Response text"}"#);
    }

    #[test]
    fn test_chat_request_deserialization() {
        let json = r#"{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Test"}]}"#;
        let request: ChatRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.model, "gpt-3.5-turbo");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert_eq!(request.messages[0].content, "Test");
    }

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{"success":true,"content":"Test response","error":null}"#;
        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.content, Some("Test response".to_string()));
        assert_eq!(response.error, None);
    }
}
