use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::Response,
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::auth::CoreAuth;
use crate::crypto;
use crate::models::TokenPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyError {
    pub error: ProxyErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
}

impl ProxyError {
    pub fn new(msg: &str, error_type: &str) -> Self {
        ProxyError {
            error: ProxyErrorDetail {
                message: msg.to_string(),
                error_type: error_type.to_string(),
            },
        }
    }

    pub fn bad_request(msg: &str) -> Self {
        Self::new(msg, "invalid_request_error")
    }

    pub fn not_found(msg: &str) -> Self {
        Self::new(msg, "not_found_error")
    }

    pub fn internal(msg: &str) -> Self {
        Self::new(msg, "server_error")
    }
}

pub fn build_target_url(base_url: &str, path: &str) -> String {
    let base_url = base_url.trim_end_matches('/');
    let target_path = if path.starts_with("/v1") {
        path.to_string()
    } else {
        format!("/v1{}", path)
    };
    format!("{}{}", base_url, target_path)
}

pub async fn chat_completions(
    State(pool): State<SqlitePool>,
    CoreAuth { api_key, .. }: CoreAuth,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, (StatusCode, Json<ProxyError>)> {
    proxy_to_token_pool(pool, api_key, method, uri, headers, body).await
}

pub async fn completions(
    State(pool): State<SqlitePool>,
    CoreAuth { api_key, .. }: CoreAuth,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, (StatusCode, Json<ProxyError>)> {
    proxy_to_token_pool(pool, api_key, method, uri, headers, body).await
}

pub async fn embeddings(
    State(pool): State<SqlitePool>,
    CoreAuth { api_key, .. }: CoreAuth,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, (StatusCode, Json<ProxyError>)> {
    proxy_to_token_pool(pool, api_key, method, uri, headers, body).await
}

pub async fn proxy_to_token_pool(
    pool: SqlitePool,
    api_key: crate::models::ApiKey,
    method: Method,
    uri: Uri,
    _headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, (StatusCode, Json<ProxyError>)> {
    let model = api_key.model.as_ref().ok_or((
        StatusCode::BAD_REQUEST,
        Json(ProxyError::bad_request("API key has no model configured")),
    ))?;

    let token_pools = sqlx::query_as::<_, TokenPool>(
        "SELECT * FROM token_pools WHERE model_type = ? AND is_active = 1"
    )
    .bind(model)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProxyError::internal("Database error")),
        )
    })?;

    let token_pool = token_pools.first().ok_or((
        StatusCode::NOT_FOUND,
        Json(ProxyError::not_found(&format!(
            "No active token pool found for model: {}",
            model
        ))),
    ))?;

    let decrypted_key = crypto::decrypt(&token_pool.api_key_encrypted).map_err(|e| {
        tracing::error!("Failed to decrypt token pool API key: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProxyError::internal("Failed to decrypt token pool credentials")),
        )
    })?;

    let target_url = build_target_url(&token_pool.base_url, uri.path());

    tracing::info!("Proxying {} {} -> {}", method, uri.path(), target_url);

    let client = Client::new();
    let mut request_builder = match method {
        Method::GET => client.get(&target_url),
        Method::POST => client.post(&target_url),
        Method::PUT => client.put(&target_url),
        Method::DELETE => client.delete(&target_url),
        Method::PATCH => client.patch(&target_url),
        _ => {
            return Err((
                StatusCode::METHOD_NOT_ALLOWED,
                Json(ProxyError::bad_request("Method not supported")),
            ))
        }
    };

    request_builder = request_builder
        .header("Authorization", format!("Bearer {}", decrypted_key))
        .header("Content-Type", "application/json");

    if !body.is_empty() {
        request_builder = request_builder.body(body);
    }

    let response = request_builder.send().await.map_err(|e| {
        tracing::error!("Proxy request failed: {}", e);
        (
            StatusCode::BAD_GATEWAY,
            Json(ProxyError::internal(&format!("Upstream request failed: {}", e))),
        )
    })?;

    let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let response_headers = response.headers().clone();
    let response_body = response.bytes().await.map_err(|e| {
        tracing::error!("Failed to read upstream response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProxyError::internal("Failed to read upstream response")),
        )
    })?;

    let mut builder = Response::builder().status(status);

    for (name, value) in response_headers.iter() {
        if name != "content-encoding" && name != "transfer-encoding" {
            builder = builder.header(name.as_str(), value.as_bytes());
        }
    }

    builder.body(Body::from(response_body)).map_err(|e| {
        tracing::error!("Failed to build response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProxyError::internal("Failed to build response")),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_error_new() {
        let error = ProxyError::new("test message", "test_type");
        assert_eq!(error.error.message, "test message");
        assert_eq!(error.error.error_type, "test_type");
    }

    #[test]
    fn test_proxy_error_bad_request() {
        let error = ProxyError::bad_request("bad request");
        assert_eq!(error.error.message, "bad request");
        assert_eq!(error.error.error_type, "invalid_request_error");
    }

    #[test]
    fn test_proxy_error_not_found() {
        let error = ProxyError::not_found("not found");
        assert_eq!(error.error.message, "not found");
        assert_eq!(error.error.error_type, "not_found_error");
    }

    #[test]
    fn test_proxy_error_internal() {
        let error = ProxyError::internal("internal error");
        assert_eq!(error.error.message, "internal error");
        assert_eq!(error.error.error_type, "server_error");
    }

    #[test]
    fn test_build_target_url_with_v1_prefix() {
        let result = build_target_url("https://api.openai.com", "/v1/chat/completions");
        assert_eq!(result, "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn test_build_target_url_without_v1_prefix() {
        let result = build_target_url("https://api.openai.com", "/chat/completions");
        assert_eq!(result, "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn test_build_target_url_trailing_slash_base() {
        let result = build_target_url("https://api.openai.com/", "/v1/chat/completions");
        assert_eq!(result, "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn test_build_target_url_empty_path() {
        let result = build_target_url("https://api.openai.com", "");
        assert_eq!(result, "https://api.openai.com/v1");
    }
}
