use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::models::ApiKey;

#[derive(Debug, Clone)]
pub struct CoreAuth {
    pub api_key: ApiKey,
    pub user_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreAuthError {
    pub error: CoreAuthErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreAuthErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
}

impl CoreAuthError {
    pub fn unauthorized(msg: &str) -> Self {
        CoreAuthError {
            error: CoreAuthErrorDetail {
                message: msg.to_string(),
                error_type: "invalid_request_error".to_string(),
            },
        }
    }
}

pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}

pub fn is_valid_api_key_format(api_key: &str) -> bool {
    api_key.starts_with("tf-")
}

pub fn verify_api_key(api_key: &str, key_hash: &str) -> bool {
    verify(api_key, key_hash).unwrap_or(false)
}

pub fn hash_api_key(api_key: &str) -> Result<String, String> {
    hash(api_key, DEFAULT_COST).map_err(|e| e.to_string())
}

#[async_trait]
impl<S> FromRequestParts<S> for CoreAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<CoreAuthError>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(CoreAuthError::unauthorized("Missing Authorization header")),
            ))?;

        let api_key = extract_bearer_token(auth_header).ok_or((
            StatusCode::UNAUTHORIZED,
            Json(CoreAuthError::unauthorized("Invalid Authorization header format")),
        ))?;

        if !is_valid_api_key_format(api_key) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(CoreAuthError::unauthorized("Invalid API key format")),
            ));
        }

        let pool = parts
            .extensions
            .get::<SqlitePool>()
            .cloned()
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreAuthError::unauthorized("Database pool not available")),
            ))?;

        let api_keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE is_active = 1"
        )
        .fetch_all(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreAuthError::unauthorized("Database error")),
            )
        })?;

        let matching_key = api_keys
            .iter()
            .find(|k| verify_api_key(api_key, &k.key_hash))
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(CoreAuthError::unauthorized("Invalid API key")),
            ))?;

        let user_id = matching_key.user_id;

        sqlx::query(
            "UPDATE api_keys SET last_used_at = datetime('now') WHERE id = ?"
        )
        .bind(matching_key.id)
        .execute(&pool)
        .await
        .ok();

        Ok(CoreAuth {
            api_key: matching_key.clone(),
            user_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unauthorized_error_format() {
        let error = CoreAuthError::unauthorized("Test error message");
        assert_eq!(error.error.message, "Test error message");
        assert_eq!(error.error.error_type, "invalid_request_error");
    }

    #[test]
    fn test_extract_bearer_token_valid() {
        let auth_header = "Bearer tf-test-key-123";
        let result = extract_bearer_token(auth_header);
        assert_eq!(result, Some("tf-test-key-123"));
    }

    #[test]
    fn test_extract_bearer_token_no_prefix() {
        let auth_header = "tf-test-key-123";
        let result = extract_bearer_token(auth_header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_bearer_token_wrong_prefix() {
        let auth_header = "Basic tf-test-key-123";
        let result = extract_bearer_token(auth_header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_is_valid_api_key_format_valid() {
        assert!(is_valid_api_key_format("tf-test-key"));
        assert!(is_valid_api_key_format("tf-"));
        assert!(is_valid_api_key_format("tf-12345"));
    }

    #[test]
    fn test_is_valid_api_key_format_invalid() {
        assert!(!is_valid_api_key_format("sk-test-key"));
        assert!(!is_valid_api_key_format("test-key"));
        assert!(!is_valid_api_key_format(""));
    }

    #[test]
    fn test_hash_and_verify_api_key() {
        let original_key = "tf-test-key-12345";
        let hashed = hash_api_key(original_key).expect("Hashing should succeed");
        
        assert_ne!(original_key, hashed, "Hash should differ from original");
        assert!(hashed.starts_with("$2b$"), "Should be bcrypt format");
        
        assert!(verify_api_key(original_key, &hashed), "Should verify correct key");
        assert!(!verify_api_key("tf-wrong-key", &hashed), "Should reject wrong key");
    }

    #[test]
    fn test_verify_api_key_invalid_hash() {
        assert!(!verify_api_key("tf-test", "invalid-hash"));
    }
}
