use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use shared::dto::service_record_dto::{
    CreateServiceRecordRequest, ServiceRecordResponse, StatusCountResponse, UpdateServiceRecordRequest,
};
use shared::error::ErrorResponse;
use crate::services::service_record_service;
use crate::AppState;

#[derive(Deserialize)]
pub struct StatusQuery {
    pub status: Option<String>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_records).post(create_record))
        .route("/counts", get(get_counts))
        .route("/{id}", get(get_record).put(update_record).delete(delete_record))
}

async fn list_records(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> Result<Json<Vec<ServiceRecordResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let result = if let Some(status) = &query.status {
        service_record_service::list_by_status(&state, status).await
    } else {
        service_record_service::list_all(&state).await
    };

    result
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))))
}

async fn get_counts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<StatusCountResponse>>, (StatusCode, Json<ErrorResponse>)> {
    service_record_service::get_status_counts(&state)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))))
}

async fn create_record(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateServiceRecordRequest>,
) -> Result<(StatusCode, Json<ServiceRecordResponse>), (StatusCode, Json<ErrorResponse>)> {
    service_record_service::create_service_record(&state, req)
        .await
        .map(|r| (StatusCode::CREATED, Json(r)))
        .map_err(|e| match &e {
            shared::error::AppError::Validation(_) =>
                (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("validation", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}

async fn get_record(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ServiceRecordResponse>, (StatusCode, Json<ErrorResponse>)> {
    let docs = service_record_service::list_all(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))))?;

    docs.into_iter()
        .find(|r| r.id == id)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", format!("Kayıt {} bulunamadı", id)))))
}

async fn update_record(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateServiceRecordRequest>,
) -> Result<Json<ServiceRecordResponse>, (StatusCode, Json<ErrorResponse>)> {
    service_record_service::update_service_record(&state, &id, req)
        .await
        .map(Json)
        .map_err(|e| match &e {
            shared::error::AppError::NotFound(_) =>
                (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", e.to_string()))),
            shared::error::AppError::Validation(_) =>
                (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("validation", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}

async fn delete_record(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    service_record_service::delete_service_record(&state, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| match &e {
            shared::error::AppError::NotFound(_) =>
                (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", e.to_string()))),
            _ =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("error", e.to_string()))),
        })
}
