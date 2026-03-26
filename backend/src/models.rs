use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Token {
    pub id: i64,
    pub name: String,
    pub symbol: String,
    pub supply: i64,
    pub created_at: String,
}
