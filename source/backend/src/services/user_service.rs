use std::sync::Arc;

use shared::dto::user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use shared::error::AppError;
use crate::AppState;
use crate::db::{sqlite, document_store};

pub async fn create_user(state: &Arc<AppState>, req: CreateUserRequest) -> Result<UserResponse, AppError> {
    if req.username.trim().is_empty() {
        return Err(AppError::Validation("Username cannot be empty".into()));
    }
    if req.email.trim().is_empty() {
        return Err(AppError::Validation("Email cannot be empty".into()));
    }

    let user = sqlite::create_user(&state.db.pool, &req)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Event log — document store'a JSON belgesi olarak kaydet
    let payload = serde_json::json!({ "user_id": user.id, "action": "created" });
    let _ = document_store::insert_document(&state.db.pool, "event_logs", &payload).await;

    Ok(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role.to_string(),
        created_at: user.created_at.to_string(),
        updated_at: user.updated_at.to_string(),
    })
}

pub async fn get_user(state: &Arc<AppState>, id: &str) -> Result<UserResponse, AppError> {
    let user = sqlite::get_user_by_id(&state.db.pool, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    Ok(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role.to_string(),
        created_at: user.created_at.to_string(),
        updated_at: user.updated_at.to_string(),
    })
}

pub async fn list_users(state: &Arc<AppState>) -> Result<Vec<UserResponse>, AppError> {
    let users = sqlite::get_all_users(&state.db.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(users.into_iter().map(|u| UserResponse {
        id: u.id,
        username: u.username,
        email: u.email,
        role: u.role.to_string(),
        created_at: u.created_at.to_string(),
        updated_at: u.updated_at.to_string(),
    }).collect())
}

pub async fn update_user(state: &Arc<AppState>, id: &str, req: UpdateUserRequest) -> Result<(), AppError> {
    let updated = sqlite::update_user(&state.db.pool, id, &req)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !updated {
        return Err(AppError::NotFound(format!("User {} not found", id)));
    }

    let payload = serde_json::json!({ "user_id": id, "action": "updated" });
    let _ = document_store::insert_document(&state.db.pool, "event_logs", &payload).await;

    Ok(())
}

pub async fn delete_user(state: &Arc<AppState>, id: &str) -> Result<(), AppError> {
    let deleted = sqlite::delete_user(&state.db.pool, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !deleted {
        return Err(AppError::NotFound(format!("User {} not found", id)));
    }

    let payload = serde_json::json!({ "user_id": id, "action": "deleted" });
    let _ = document_store::insert_document(&state.db.pool, "event_logs", &payload).await;

    Ok(())
}
