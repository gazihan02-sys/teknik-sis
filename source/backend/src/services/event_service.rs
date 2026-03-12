use std::sync::Arc;
use crate::AppState;
use crate::db::document_store;

#[allow(dead_code)]
pub async fn log_custom_event(state: &Arc<AppState>, event_type: &str, payload: &serde_json::Value) {
    let event = serde_json::json!({
        "event_type": event_type,
        "payload": payload,
    });
    let _ = document_store::insert_document(&state.db.pool, "event_logs", &event).await;
}
