use axum::{
    routing::{get, post},
    Router,
    extract::DefaultBodyLimit,
};
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tokenforest_backend::core::proxy::{chat_completions, completions, embeddings};
use tokenforest_backend::db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "dev".to_string());
    let env_file = format!(".{}.env", run_mode);
    dotenvy::from_path_override(&env_file).ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tokenforest_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./database/tokenforest.db?mode=rwc".to_string());
    
    let pool = SqlitePool::connect(&database_url).await?;
    tracing::info!("Core server connected to SQLite database");

    db::migrate(&pool).await?;

    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/completions", post(completions))
        .route("/v1/embeddings", post(embeddings))
        .route("/v1/models", get(models_list))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(pool);

    let host: std::net::IpAddr = std::env::var("CORE_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        .parse()
        .unwrap_or(([0, 0, 0, 0]).into());
    let port: u16 = std::env::var("CORE_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .unwrap_or(8000);
    let addr = SocketAddr::from((host, port));
    tracing::info!("Core server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn models_list() -> &'static str {
    r#"{"object":"list","data":[]}"#
}
