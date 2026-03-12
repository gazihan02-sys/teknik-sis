use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use shared::dto::user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use shared::error::ErrorResponse;
use crate::services::user_service;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
}

async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, Json<ErrorResponse>)> {
    user_service::list_users(&state)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorResponse>)> {
    user_service::create_user(&state, req)
        .await
        .map(|u| (StatusCode::CREATED, Json(u)))
        .map_err(|e| match &e {
            shared::error::AppError::Validation(_) =>
                (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("validation", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ErrorResponse>)> {
    user_service::get_user(&state, &id)
        .await
        .map(Json)
        .map_err(|e| match &e {
            shared::error::AppError::NotFound(_) =>
                (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    user_service::update_user(&state, &id, req)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| match &e {
            shared::error::AppError::NotFound(_) =>
                (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    user_service::delete_user(&state, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| match &e {
            shared::error::AppError::NotFound(_) =>
                (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}
