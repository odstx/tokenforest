use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::CurrentUser;
use super::{AppPool, ErrorResponse};

#[derive(Serialize, ToSchema)]
pub struct StatsResponse {
    pub api_keys_count: i64,
    pub token_pools_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/stats",
    responses(
        (status = 200, description = "User statistics", body = StatsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_stats(
    State(pool): State<AppPool>,
    CurrentUser(claims): CurrentUser,
) -> Result<Json<StatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let api_keys_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE user_id = ?")
        .bind(claims.sub)
        .fetch_one(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to count API keys".to_string(),
                }),
            )
        })?;

    let token_pools_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM token_pools WHERE user_id = ?")
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

    Ok(Json(StatsResponse {
        api_keys_count,
        token_pools_count,
    }))
}
