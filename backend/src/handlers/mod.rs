use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

pub mod api_keys;
pub mod auth;
pub mod stats;
pub mod token_pools;

pub use api_keys::*;
pub use auth::*;
pub use stats::*;
pub use token_pools::*;

pub type AppPool = SqlitePool;

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}
