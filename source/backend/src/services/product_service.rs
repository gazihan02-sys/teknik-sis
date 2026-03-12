use std::sync::Arc;

use shared::dto::product_dto::{CreateProductRequest, ProductResponse};
use shared::error::AppError;
use crate::AppState;
use crate::db::document_store;

const COLLECTION: &str = "products";

pub async fn create_product(state: &Arc<AppState>, req: CreateProductRequest) -> Result<ProductResponse, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("Product name cannot be empty".into()));
    }

    // Ürünü JSON belgesi olarak kaydet
    let data = serde_json::json!({
        "name": req.name,
        "description": req.description,
        "price": req.price,
        "stock": req.stock,
        "category": req.category,
    });

    let id = document_store::insert_document(&state.db.pool, COLLECTION, &data)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Event log
    let event = serde_json::json!({ "product_id": id, "action": "created" });
    let _ = document_store::insert_document(&state.db.pool, "event_logs", &event).await;

    let doc = document_store::find_by_id(&state.db.pool, COLLECTION, &id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::Internal("Failed to read back created document".into()))?;

    Ok(doc_to_response(&doc))
}

pub async fn list_products(state: &Arc<AppState>) -> Result<Vec<ProductResponse>, AppError> {
    let docs = document_store::find_all(&state.db.pool, COLLECTION)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(docs.iter().map(doc_to_response).collect())
}

pub async fn get_product(state: &Arc<AppState>, id: &str) -> Result<ProductResponse, AppError> {
    document_store::find_by_id(&state.db.pool, COLLECTION, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .map(|d| doc_to_response(&d))
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))
}

pub async fn delete_product(state: &Arc<AppState>, id: &str) -> Result<(), AppError> {
    let deleted = document_store::delete_document(&state.db.pool, COLLECTION, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !deleted {
        return Err(AppError::NotFound(format!("Product {} not found", id)));
    }

    let event = serde_json::json!({ "product_id": id, "action": "deleted" });
    let _ = document_store::insert_document(&state.db.pool, "event_logs", &event).await;

    Ok(())
}

fn doc_to_response(doc: &document_store::Document) -> ProductResponse {
    let d = &doc.data;
    ProductResponse {
        id: doc.id.clone(),
        name: d["name"].as_str().unwrap_or_default().to_string(),
        description: d["description"].as_str().unwrap_or_default().to_string(),
        price: d["price"].as_f64().unwrap_or_default(),
        stock: d["stock"].as_i64().unwrap_or_default() as i32,
        category: d["category"].as_str().unwrap_or_default().to_string(),
        created_at: doc.created_at.clone(),
        updated_at: doc.updated_at.clone(),
    }
}
