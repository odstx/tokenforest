use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::models::Token;

pub type AppPool = SqlitePool;

/// GET / - Index page
pub async fn index() -> &'static str {
    "Welcome to TokenForest API 🌲"
}

/// GET /api/tokens - List all tokens
pub async fn list_tokens(
    State(pool): State<AppPool>,
) -> Result<Json<Vec<Token>>, StatusCode> {
    let tokens = sqlx::query_as::<_, Token>("SELECT * FROM tokens ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tokens))
}

/// POST /api/tokens - Create a new token
pub async fn create_token(
    State(pool): State<AppPool>,
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<Token>, StatusCode> {
    let token = sqlx::query_as::<_, Token>(
        "INSERT INTO tokens (name, symbol, supply, created_at) VALUES (?, ?, ?, datetime('now')) RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.symbol)
    .bind(&payload.supply)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(token))
}

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub symbol: String,
    pub supply: i64,
}
