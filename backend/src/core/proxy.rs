use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::Response,
    Json,
};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::auth::CoreAuth;
use super::metrics;
use crate::crypto;

use crate::models::TokenPool;

static CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();

fn get_client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .tcp_nodelay(true)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client")
    })
}

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

#[derive(Debug, Deserialize)]
struct ChatRequest {
    #[allow(dead_code)]
    model: Option<String>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    usage: Option<Usage>,
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
    let start_time = std::time::Instant::now();
    
    let model = api_key.model.as_ref().ok_or((
        StatusCode::BAD_REQUEST,
        Json(ProxyError::bad_request("API key has no model configured")),
    ))?;

    let chat_request: Option<ChatRequest> = serde_json::from_slice(&body).ok();
    let is_stream = chat_request.as_ref().and_then(|r| r.stream).unwrap_or(false);

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
    
    tracing::info!("[TIMING] DB lookup took {:?}", start_time.elapsed());

    tracing::debug!(
        "Selected token pool ID: {}, name: {}, api_key_encrypted length: {}",
        token_pool.id,
        token_pool.name,
        token_pool.api_key_encrypted.len()
    );

    let decrypted_key = crypto::decrypt(&token_pool.api_key_encrypted).map_err(|e| {
        tracing::error!(
            "Failed to decrypt token pool API key for pool ID {}: {}, encrypted value prefix: {:?}",
            token_pool.id,
            e,
            &token_pool.api_key_encrypted.chars().take(20).collect::<String>()
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProxyError::internal("Failed to decrypt token pool credentials")),
        )
    })?;

    let target_url = build_target_url(&token_pool.base_url, uri.path());

    tracing::info!("Proxying {} {} -> {}", method, uri.path(), target_url);

    let client = get_client();
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
        request_builder = request_builder.body(body.clone());
    }

    let response = request_builder.send().await.map_err(|e| {
        tracing::error!("Proxy request failed: {}", e);
        metrics::record_request(model, &api_key.id.to_string(), "error");
        (
            StatusCode::BAD_GATEWAY,
            Json(ProxyError::internal(&format!("Upstream request failed: {}", e))),
        )
    })?;
    
    tracing::info!("[TIMING] Upstream response headers received in {:?}", start_time.elapsed());

    let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let response_headers = response.headers().clone();

    if is_stream {
        let model = model.clone();
        let api_key_id = api_key.id.to_string();
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(100);
        let stream_start = start_time;
        
        let mut response_stream = response.bytes_stream();
        let mut total_input = 0u64;
        let mut total_output = 0u64;
        let mut first_chunk_logged = false;
        
        tokio::spawn(async move {
            while let Some(chunk_result) = response_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if !first_chunk_logged {
                            tracing::info!("[TIMING] First chunk received at {:?}", stream_start.elapsed());
                            first_chunk_logged = true;
                        }
                        let (input, output) = parse_stream_usage(&chunk);
                        total_input += input;
                        total_output += output;
                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Stream error: {}", e);
                        let _ = tx.send(Err(std::io::Error::other(e))).await;
                        break;
                    }
                }
            }
            
            tracing::info!("[TIMING] Stream completed in {:?}", stream_start.elapsed());
            
            if total_input > 0 || total_output > 0 {
                metrics::record_tokens(&model, &api_key_id, total_input, total_output);
            }
            metrics::record_request(&model, &api_key_id, "success");
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        let body = Body::from_stream(stream);
        
        let mut builder = Response::builder().status(status);
        for (name, value) in response_headers.iter() {
            if name != "content-encoding" && name != "transfer-encoding" {
                builder = builder.header(name.as_str(), value.as_bytes());
            }
        }
        
        return builder.body(body).map_err(|e| {
            tracing::error!("Failed to build response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProxyError::internal("Failed to build response")),
            )
        });
    }

    let response_body = response.bytes().await.map_err(|e| {
        tracing::error!("Failed to read upstream response: {}", e);
        metrics::record_request(model, &api_key.id.to_string(), "error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProxyError::internal("Failed to read upstream response")),
        )
    })?;

    if status.is_success() {
        let (input_tokens, output_tokens) = parse_non_stream_usage(&response_body);
        
        if input_tokens > 0 || output_tokens > 0 {
            metrics::record_tokens(model, &api_key.id.to_string(), input_tokens, output_tokens);
        }
        metrics::record_request(model, &api_key.id.to_string(), "success");
    } else {
        metrics::record_request(model, &api_key.id.to_string(), "error");
    }

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

fn parse_non_stream_usage(body: &[u8]) -> (u64, u64) {
    let response: ChatResponse = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => return (0, 0),
    };
    
    match response.usage {
        Some(usage) => (usage.prompt_tokens, usage.completion_tokens),
        None => (0, 0),
    }
}

fn parse_stream_usage(body: &[u8]) -> (u64, u64) {
    let body_str = match std::str::from_utf8(body) {
        Ok(s) => s,
        Err(_) => return (0, 0),
    };
    
    for line in body_str.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                continue;
            }
            if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                if let Some(usage) = chunk.usage {
                    return (usage.prompt_tokens, usage.completion_tokens);
                }
            }
        }
    }
    (0, 0)
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
