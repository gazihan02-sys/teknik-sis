use std::sync::Arc;
use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "sis-teknik-backend",
        "version": "0.1.0"
    }))
}
