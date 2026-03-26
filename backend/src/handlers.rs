use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::auth::{create_jwt_token, CurrentUser};
use crate::models::{Token, User};

pub type AppPool = SqlitePool;

pub async fn index() -> &'static str {
    "Welcome to TokenForest API"
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

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

pub async fn list_tokens(
    State(pool): State<AppPool>,
    _user: CurrentUser,
) -> Result<Json<Vec<Token>>, StatusCode> {
    let tokens = sqlx::query_as::<_, Token>("SELECT * FROM tokens ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tokens))
}

pub async fn create_token(
    State(pool): State<AppPool>,
    _user: CurrentUser,
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
