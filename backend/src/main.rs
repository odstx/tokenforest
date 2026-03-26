use axum::{
    routing::{get, post},
    Router,
};
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod handlers;
mod models;
mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tokenforest_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./database/tokenforest.db?mode=rwc".to_string());
    
    let pool = SqlitePool::connect(&database_url).await?;
    tracing::info!("Connected to SQLite database");

    db::migrate(&pool).await?;

    let token_routes = Router::new()
        .route("/", get(handlers::list_tokens))
        .route("/", post(handlers::create_token))
        .route_layer(axum::middleware::from_fn(auth::auth_middleware));

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .nest("/api/tokens", token_routes)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
