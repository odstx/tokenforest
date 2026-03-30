use axum::{
    routing::{get, post, put},
    Router,
    middleware,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    body::Body,
};
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use rust_embed::RustEmbed;

mod auth;
#[allow(dead_code)]
mod core;
mod handlers;
mod models;
mod db;
mod crypto;

#[derive(RustEmbed)]
#[folder = "../frontend/build/"]
struct FrontendAssets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    
    if path.is_empty() || path == "index.html" {
        path = "index.html".to_string();
    }
    
    match FrontendAssets::get(&path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .as_ref()
                .to_string();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .body::<Body>(content.data.into_owned().into())
                .unwrap()
        }
        None => match FrontendAssets::get("index.html") {
                Some(content) => {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body::<Body>(content.data.into_owned().into())
                        .unwrap()
                }
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body::<Body>("Not found".into())
                    .unwrap(),
        },
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
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
        handlers::chat_with_token_pool,
        handlers::get_api_key,
        handlers::get_raw_api_key,
        handlers::get_token_pool,
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
            handlers::ChatRequest,
            handlers::ChatResponse,
            handlers::StatsResponse,
            handlers::RawKeyResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "api-keys", description = "API key management"),
        (name = "token-pools", description = "Token pool management")
    )
)]
struct ApiDoc;

fn check_required_env_vars() {
    let required_vars = ["JWT_SECRET", "ENCRYPTION_KEY"];
    let missing: Vec<&str> = required_vars
        .iter()
        .filter(|var| std::env::var(var).is_err())
        .copied()
        .collect();
    
    if !missing.is_empty() {
        eprintln!("\n❌ Missing required environment variables:\n");
        for var in &missing {
            eprintln!("   - {}", var);
        }
        eprintln!("\n📝 Please set them in one of the following ways:\n");
        eprintln!("   1. Create a .dev.env file in the project root with:");
        for var in &missing {
            eprintln!("      {}=your-value-here", var);
        }
        eprintln!("\n   2. Or set them directly in your shell:");
        for var in &missing {
            eprintln!("      export {}=your-value-here", var);
        }
        eprintln!("\n   Example values:");
        eprintln!("      JWT_SECRET=your-secret-key-at-least-32-characters-long");
        eprintln!("      ENCRYPTION_KEY=your-encryption-key-at-least-32-characters\n");
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "dev".to_string());
    let env_file = format!(".{}.env", run_mode);
    dotenvy::from_path_override(&env_file).ok();
    
    check_required_env_vars();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tokenforest_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:/opt/tf/database/tokenforest.db?mode=rwc".to_string());
    
    let pool = SqlitePool::connect(&database_url).await?;
    tracing::info!("Connected to SQLite database");

    db::migrate(&pool).await?;

    let protected_routes = Router::new()
        .route("/api/stats", get(handlers::get_stats))
        .route("/api/api-keys", get(handlers::list_api_keys).post(handlers::create_api_key))
        .route("/api/api-keys/chat", post(handlers::chat_with_api_key))
        .route("/api/api-keys/:id/key", get(handlers::get_raw_api_key))
        .route("/api/api-keys/:id", get(handlers::get_api_key).put(handlers::update_api_key).delete(handlers::delete_api_key))
        .route("/api/api-keys/:id/toggle", put(handlers::toggle_api_key))
        .route("/api/api-keys/:id/test", post(handlers::test_api_key))
        .route("/api/token-pools", get(handlers::list_token_pools).post(handlers::create_token_pool))
        .route("/api/token-pools/:id", get(handlers::get_token_pool).put(handlers::update_token_pool).delete(handlers::delete_token_pool))
        .route("/api/token-pools/:id/toggle", put(handlers::toggle_token_pool))
        .route("/api/token-pools/:id/test", post(handlers::test_token_pool))
        .route("/api/token-pools/:id/chat", post(handlers::chat_with_token_pool))
        .route_layer(middleware::from_fn(auth::auth_middleware));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/config", get(handlers::get_config))
        .merge(protected_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback(static_handler)
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
