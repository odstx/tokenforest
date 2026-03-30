use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ConfigResponse {
    pub base_url: String,
    pub api_version: String,
}

#[utoipa::path(
    get,
    path = "/api/config",
    responses(
        (status = 200, description = "Configuration", body = ConfigResponse)
    )
)]
pub async fn get_config() -> axum::Json<ConfigResponse> {
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8000/v1".to_string());
    
    axum::Json(ConfigResponse {
        base_url,
        api_version: "v1".to_string(),
    })
}