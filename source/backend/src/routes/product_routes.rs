use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use shared::dto::product_dto::{CreateProductRequest, ProductResponse};
use shared::error::ErrorResponse;
use crate::services::product_service;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_products).post(create_product))
        .route("/{id}", get(get_product).delete(delete_product))
}

async fn list_products(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProductResponse>>, (StatusCode, Json<ErrorResponse>)> {
    product_service::list_products(&state)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))))
}

async fn create_product(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ProductResponse>), (StatusCode, Json<ErrorResponse>)> {
    product_service::create_product(&state, req)
        .await
        .map(|p| (StatusCode::CREATED, Json(p)))
        .map_err(|e| match &e {
            shared::error::AppError::Validation(_) =>
                (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("validation", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}

async fn get_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ProductResponse>, (StatusCode, Json<ErrorResponse>)> {
    product_service::get_product(&state, &id)
        .await
        .map(Json)
        .map_err(|e| match &e {
            shared::error::AppError::NotFound(_) =>
                (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}

async fn delete_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    product_service::delete_product(&state, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| match &e {
            shared::error::AppError::NotFound(_) =>
                (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}
