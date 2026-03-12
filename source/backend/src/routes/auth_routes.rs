use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use axum::http::HeaderMap;

use shared::dto::auth_dto::{LoginRequest, LoginResponse, MeResponse};
use shared::error::ErrorResponse;
use crate::services::auth_service;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/me", post(me))
        .route("/logout", post(logout_handler))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    auth_service::login(&state, req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("auth_error", e.to_string()))))
}

async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<MeResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim_start_matches("Bearer "))
        .unwrap_or("");

    auth_service::me(&state, token)
        .await
        .map(Json)
        .map_err(|_| (StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("auth_error", "Geçersiz oturum"))))
}

async fn logout_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim_start_matches("Bearer "))
        .unwrap_or("");

    auth_service::logout(&state, token)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))))
}
