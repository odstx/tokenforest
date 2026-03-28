use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApiKey {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub model: Option<String>,
    pub key_hash: String,
    pub prefix: String,
    pub is_active: i32,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub model: Option<String>,
    pub prefix: String,
    pub is_active: bool,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub model: Option<String>,
    pub key: String,
    pub prefix: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    10
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TokenPool {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub model_type: String,
    pub base_url: String,
    pub api_key_encrypted: String,
    pub is_active: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenPoolResponse {
    pub id: i64,
    pub name: String,
    pub model_type: String,
    pub base_url: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TokenPool> for TokenPoolResponse {
    fn from(pool: TokenPool) -> Self {
        TokenPoolResponse {
            id: pool.id,
            name: pool.name,
            model_type: pool.model_type,
            base_url: pool.base_url,
            is_active: pool.is_active != 0,
            created_at: pool.created_at,
            updated_at: pool.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTokenPoolRequest {
    pub name: String,
    pub model_type: String,
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTokenPoolRequest {
    pub name: Option<String>,
    pub model_type: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}
