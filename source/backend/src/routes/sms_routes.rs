use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};

use shared::dto::sms_log_dto::SmsLogResponse;
use shared::error::ErrorResponse;
use crate::services::sms_service;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_sms_logs))
}

async fn list_sms_logs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SmsLogResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let docs = sms_service::list_sms_logs(&state.db.pool).await;
    let logs: Vec<SmsLogResponse> = docs.iter().map(|doc| {
        let d = &doc.data;
        SmsLogResponse {
            id: doc.id.clone(),
            record_id: d["record_id"].as_str().unwrap_or_default().to_string(),
            status: d["status"].as_str().unwrap_or_default().to_string(),
            phone: d["phone"].as_str().unwrap_or_default().to_string(),
            customer_name: d["customer_name"].as_str().unwrap_or_default().to_string(),
            content: d["content"].as_str().unwrap_or_default().to_string(),
            created_at: doc.created_at.clone(),
        }
    }).collect();
    Ok(Json(logs))
}
