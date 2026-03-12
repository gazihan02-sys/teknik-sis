mod config;
mod db;
mod routes;
mod services;

use std::sync::Arc;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::AppConfig;
use crate::db::Database;

pub struct AppState {
    pub db: Database,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() {
    // Load .env
    dotenvy::dotenv().ok();

    // Tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Config
    let config = AppConfig::from_env();
    let addr = format!("{}:{}", config.backend_host, config.backend_port);

    // Database
    let db = Database::new(&config).await.expect("Failed to initialize database");
    db.run_migrations().await.expect("Failed to run migrations");

    tracing::info!("SQLite initialized (relational + JSON document store)");

    // Shared state
    let state = Arc::new(AppState { db, config });

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Router
    let app = Router::new()
        .nest("/api", routes::api_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!("Backend server starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
