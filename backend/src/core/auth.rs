use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::net::IpAddr;

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
    let key_without_prefix = api_key.strip_prefix("tf-").unwrap_or(api_key);
    verify(key_without_prefix, key_hash).unwrap_or(false)
}

pub fn hash_api_key(api_key: &str) -> Result<String, String> {
    hash(api_key, DEFAULT_COST).map_err(|e| e.to_string())
}

pub fn extract_client_ip(parts: &Parts) -> Option<String> {
    if let Some(forwarded) = parts.headers.get("X-Forwarded-For").and_then(|h| h.to_str().ok()) {
        if let Some(first_ip) = forwarded.split(',').next() {
            return Some(first_ip.trim().to_string());
        }
    }
    parts
        .headers
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

pub fn is_ip_allowed(client_ip: &str, allowed_cidrs: &[String]) -> bool {
    let client: IpAddr = match client_ip.parse() {
        Ok(ip) => ip,
        Err(_) => return false,
    };
    for cidr_str in allowed_cidrs {
        if let Ok(cidr) = cidr_str.parse::<IpNetwork>() {
            if cidr.contains(client) {
                return true;
            }
        }
    }
    false
}

#[async_trait]
impl FromRequestParts<SqlitePool> for CoreAuth
{
    type Rejection = (StatusCode, Json<CoreAuthError>);

    async fn from_request_parts(parts: &mut Parts, pool: &SqlitePool) -> Result<Self, Self::Rejection> {
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

        let pool = pool.clone();

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

        if let Some(ref cidrs_json) = matching_key.allowed_cidrs {
            let allowed_cidrs: Vec<String> =
                serde_json::from_str(cidrs_json).unwrap_or_default();
            if !allowed_cidrs.is_empty() {
                let client_ip =
                    extract_client_ip(parts).ok_or((
                        StatusCode::UNAUTHORIZED,
                        Json(CoreAuthError::unauthorized("Unable to determine client IP")),
                    ))?;
                if !is_ip_allowed(&client_ip, &allowed_cidrs) {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(CoreAuthError::unauthorized("IP address not allowed")),
                    ));
                }
            }
        }

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
        let raw_key = "test-key-12345";
        let hashed = hash_api_key(raw_key).expect("Hashing should succeed");

        assert_ne!(raw_key, hashed, "Hash should differ from original");
        assert!(hashed.starts_with("$2b$"), "Should be bcrypt format");

        assert!(
            verify_api_key(&format!("tf-{}", raw_key), &hashed),
            "Should verify correct key with tf- prefix"
        );
        assert!(!verify_api_key("tf-wrong-key", &hashed), "Should reject wrong key");
    }

    #[test]
    fn test_verify_api_key_invalid_hash() {
        assert!(!verify_api_key("tf-test", "invalid-hash"));
    }

    #[test]
    fn test_is_ip_allowed_single_ip() {
        assert!(is_ip_allowed("192.168.1.1", &["192.168.1.1".to_string()]));
        assert!(is_ip_allowed("10.0.0.1", &["10.0.0.1".to_string()]));
        assert!(!is_ip_allowed("192.168.1.2", &["192.168.1.1".to_string()]));
    }

    #[test]
    fn test_is_ip_allowed_cidr() {
        assert!(is_ip_allowed("192.168.1.100", &["192.168.1.0/24".to_string()]));
        assert!(is_ip_allowed("192.168.1.0", &["192.168.1.0/24".to_string()]));
        assert!(is_ip_allowed("192.168.1.255", &["192.168.1.0/24".to_string()]));
        assert!(!is_ip_allowed("192.168.2.1", &["192.168.1.0/24".to_string()]));
    }

    #[test]
    fn test_is_ip_allowed_multiple_cidrs() {
        let cidrs = vec!["192.168.1.0/24".to_string(), "10.0.0.0/8".to_string()];
        assert!(is_ip_allowed("192.168.1.50", &cidrs));
        assert!(is_ip_allowed("10.255.255.255", &cidrs));
        assert!(!is_ip_allowed("172.16.0.1", &cidrs));
    }

    #[test]
    fn test_is_ip_allowed_ipv6() {
        assert!(is_ip_allowed("::1", &["::1/128".to_string()]));
        assert!(is_ip_allowed("2001:db8::1", &["2001:db8::/32".to_string()]));
    }

    #[test]
    fn test_is_ip_allowed_invalid_ip() {
        assert!(!is_ip_allowed("invalid", &["192.168.1.0/24".to_string()]));
    }

    #[test]
    fn test_is_ip_allowed_empty_cidrs() {
        assert!(!is_ip_allowed("192.168.1.1", &[]));
    }
}
