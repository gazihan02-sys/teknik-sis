use std::sync::Arc;

use sha2::{Sha256, Digest};
use shared::dto::auth_dto::{LoginRequest, LoginResponse, MeResponse};
use shared::error::AppError;
use crate::AppState;
use crate::db::{sqlite, document_store};

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn login(state: &Arc<AppState>, req: LoginRequest) -> Result<LoginResponse, AppError> {
    let result = sqlite::get_user_by_username(&state.db.pool, &req.username)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let (user, stored_hash) = result
        .ok_or_else(|| AppError::Validation("Kullanıcı adı veya şifre hatalı".into()))?;

    let input_hash = hash_password(&req.password);
    if input_hash != stored_hash {
        return Err(AppError::Validation("Kullanıcı adı veya şifre hatalı".into()));
    }

    // Oturum token'ı oluştur
    let token = uuid::Uuid::new_v4().to_string();

    // Token'ı sessions koleksiyonuna kaydet
    let session_data = serde_json::json!({
        "token": token,
        "user_id": user.id,
        "username": user.username,
        "role": user.role.to_string(),
    });
    document_store::insert_document(&state.db.pool, "sessions", &session_data)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(LoginResponse {
        token,
        username: user.username,
        role: user.role.to_string(),
    })
}

pub async fn me(state: &Arc<AppState>, token: &str) -> Result<MeResponse, AppError> {
    let docs = document_store::find_by_field(&state.db.pool, "sessions", "token", token)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if let Some(doc) = docs.first() {
        let username = doc.data.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let role = doc.data.get("role").and_then(|v| v.as_str()).unwrap_or("user").to_string();
        return Ok(MeResponse { username, role });
    }

    Err(AppError::Validation("Geçersiz oturum".into()))
}

pub async fn logout(state: &Arc<AppState>, token: &str) -> Result<(), AppError> {
    let docs = document_store::find_by_field(&state.db.pool, "sessions", "token", token)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if let Some(doc) = docs.first() {
        let _ = document_store::delete_document(&state.db.pool, "sessions", &doc.id).await;
    }

    Ok(())
}
