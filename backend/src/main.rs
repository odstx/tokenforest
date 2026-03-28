use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod auth;
mod handlers;
mod models;
mod db;
mod crypto;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::index,
        handlers::register,
        handlers::login,
        handlers::list_api_keys,
        handlers::create_api_key,
        handlers::delete_api_key,
        handlers::toggle_api_key,
        handlers::list_token_pools,
        handlers::create_token_pool,
        handlers::update_token_pool,
        handlers::delete_token_pool,
        handlers::toggle_token_pool,
        handlers::test_token_pool,
        handlers::get_stats
    ),
    components(
        schemas(
            models::User,
            models::ApiKeyResponse,
            models::CreateApiKeyRequest,
            models::CreateApiKeyResponse,
            models::TokenPool,
            models::TokenPoolResponse,
            models::CreateTokenPoolRequest,
            models::UpdateTokenPoolRequest,
            handlers::RegisterRequest,
            handlers::LoginRequest,
            handlers::AuthResponse,
            handlers::UserInfo,
            handlers::ErrorResponse,
            handlers::TestTokenPoolResponse,
            handlers::StatsResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "api-keys", description = "API key management"),
        (name = "token-pools", description = "Token pool management")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "dev".to_string());
    let env_file = format!(".{}.env", run_mode);
    dotenvy::from_path_override(&env_file).ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tokenforest_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./database/tokenforest.db?mode=rwc".to_string());
    
    let pool = SqlitePool::connect(&database_url).await?;
    tracing::info!("Connected to SQLite database");

    db::migrate(&pool).await?;

    let protected_routes = Router::new()
        .route("/api/stats", get(handlers::get_stats))
        .route("/api/api-keys", get(handlers::list_api_keys).post(handlers::create_api_key))
        .route("/api/api-keys/:id", delete(handlers::delete_api_key))
        .route("/api/api-keys/:id/toggle", put(handlers::toggle_api_key))
        .route("/api/token-pools", get(handlers::list_token_pools).post(handlers::create_token_pool))
        .route("/api/token-pools/:id", put(handlers::update_token_pool).delete(handlers::delete_token_pool))
        .route("/api/token-pools/:id/toggle", put(handlers::toggle_token_pool))
        .route("/api/token-pools/:id/test", post(handlers::test_token_pool))
        .route_layer(middleware::from_fn(auth::auth_middleware));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .merge(protected_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .with_state(pool);

    let host: std::net::IpAddr = std::env::var("HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        .parse()
        .unwrap_or(([0, 0, 0, 0]).into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);
    let addr = SocketAddr::from((host, port));
    tracing::info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
