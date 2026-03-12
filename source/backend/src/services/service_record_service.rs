use std::sync::Arc;

use shared::dto::service_record_dto::{
    CreateServiceRecordRequest, ServiceRecordResponse, UpdateServiceRecordRequest, StatusCountResponse,
};
use shared::error::AppError;
use crate::AppState;
use crate::db::document_store;

const COLLECTION: &str = "service_records";

/// Türkçe karakter desteğiyle büyük harfe çevirir.
/// i → İ, ı → I, ö → Ö, ü → Ü, ş → Ş, ç → Ç, ğ → Ğ
fn turkish_uppercase(s: &str) -> String {
    s.chars().map(|c| match c {
        'i' => 'İ',
        'ı' => 'I',
        'ö' => 'Ö',
        'ü' => 'Ü',
        'ş' => 'Ş',
        'ç' => 'Ç',
        'ğ' => 'Ğ',
        other => {
            let mut upper = other.to_uppercase();
            upper.next().unwrap_or(other)
        }
    }).collect()
}

const STATUSES: &[&str] = &[
    "musteri_kabul",
    "teknisyene_verildi",
    "islem_bekliyor",
    "parca_bekliyor",
    "merkeze_sevk",
    "degisim",
    "tamir_tamamlandi",
    "teslim_edildi",
    "iade",
];

pub async fn create_service_record(
    state: &Arc<AppState>,
    req: CreateServiceRecordRequest,
) -> Result<ServiceRecordResponse, AppError> {
    if req.customer_name.trim().is_empty() {
        return Err(AppError::Validation("Müşteri adı boş olamaz".into()));
    }
    if req.issue.trim().is_empty() {
        return Err(AppError::Validation("Müşteri şikayeti boş olamaz".into()));
    }

    let data = serde_json::json!({
        "customer_name": turkish_uppercase(req.customer_name.trim()),
        "phone": req.phone.trim(),
        "device": turkish_uppercase(req.device.trim()),
        "brand": turkish_uppercase(req.brand.trim()),
        "model": turkish_uppercase(req.model.trim()),
        "serial_number": turkish_uppercase(req.serial_number.trim()),
        "issue": turkish_uppercase(req.issue.trim()),
        "notes": turkish_uppercase(req.notes.trim()),
        "accessories": turkish_uppercase(req.accessories.trim()),
        "doc_fatura": req.doc_fatura,
        "doc_garanti": req.doc_garanti,
        "doc_uretim": req.doc_uretim,
        "doc_ariza": req.doc_ariza,
        "status": "musteri_kabul",
    });

    let id = document_store::insert_document(&state.db.pool, COLLECTION, &data)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let doc = document_store::find_by_id(&state.db.pool, COLLECTION, &id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::Internal("Kayıt oluşturulamadı".into()))?;

    // SMS gönder (arka planda, hata olsa bile kayıt engellenmez)
    if state.config.sms.enabled {
        let sms_config = &state.config.sms;
        let phone = req.phone.clone();
        let name = turkish_uppercase(req.customer_name.trim());
        let brand_model = format!("{} {}", turkish_uppercase(req.brand.trim()), turkish_uppercase(req.model.trim()));
        let sn = turkish_uppercase(req.serial_number.trim());
        let sms_base = sms_config.base_url.clone();
        let sms_user = sms_config.username.clone();
        let sms_pass = sms_config.password.clone();
        let sms_sender = sms_config.sender.clone();
        let pool = state.db.pool.clone();
        let record_id = id.clone();
        tokio::spawn(async move {
            let cfg = crate::config::SmsConfig {
                base_url: sms_base,
                username: sms_user,
                password: sms_pass,
                sender: sms_sender,
                enabled: true,
            };
            crate::services::sms_service::send_status_sms(&cfg, &pool, &record_id, &phone, &name, &brand_model, &sn, "musteri_kabul").await;
        });
    }

    Ok(doc_to_response(&doc))
}

pub async fn list_by_status(
    state: &Arc<AppState>,
    status: &str,
) -> Result<Vec<ServiceRecordResponse>, AppError> {
    let docs = document_store::find_by_field(&state.db.pool, COLLECTION, "status", status)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(docs.iter().map(doc_to_response).collect())
}

pub async fn list_all(
    state: &Arc<AppState>,
) -> Result<Vec<ServiceRecordResponse>, AppError> {
    let docs = document_store::find_all(&state.db.pool, COLLECTION)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(docs.iter().map(doc_to_response).collect())
}

pub async fn get_status_counts(
    state: &Arc<AppState>,
) -> Result<Vec<StatusCountResponse>, AppError> {
    let mut counts = Vec::new();
    for &status in STATUSES {
        let docs = document_store::find_by_field(&state.db.pool, COLLECTION, "status", status)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        counts.push(StatusCountResponse {
            status: status.to_string(),
            count: docs.len() as i64,
        });
    }
    Ok(counts)
}

pub async fn update_service_record(
    state: &Arc<AppState>,
    id: &str,
    req: UpdateServiceRecordRequest,
) -> Result<ServiceRecordResponse, AppError> {
    let existing = document_store::find_by_id(&state.db.pool, COLLECTION, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Kayıt {} bulunamadı", id)))?;

    let d = &existing.data;
    let new_status = req.status.as_deref().unwrap_or(d["status"].as_str().unwrap_or_default());
    let old_status = d["status"].as_str().unwrap_or_default();
    let status_changed = req.status.is_some() && new_status != old_status;

    let data = serde_json::json!({
        "customer_name": turkish_uppercase(&req.customer_name.unwrap_or_else(|| d["customer_name"].as_str().unwrap_or_default().to_string())),
        "phone": req.phone.unwrap_or_else(|| d["phone"].as_str().unwrap_or_default().to_string()),
        "device": turkish_uppercase(&req.device.unwrap_or_else(|| d["device"].as_str().unwrap_or_default().to_string())),
        "brand": turkish_uppercase(&req.brand.unwrap_or_else(|| d["brand"].as_str().unwrap_or_default().to_string())),
        "model": turkish_uppercase(&req.model.unwrap_or_else(|| d["model"].as_str().unwrap_or_default().to_string())),
        "serial_number": turkish_uppercase(&req.serial_number.unwrap_or_else(|| d["serial_number"].as_str().unwrap_or_default().to_string())),
        "issue": turkish_uppercase(&req.issue.unwrap_or_else(|| d["issue"].as_str().unwrap_or_default().to_string())),
        "notes": turkish_uppercase(&req.notes.unwrap_or_else(|| d["notes"].as_str().unwrap_or_default().to_string())),
        "accessories": turkish_uppercase(&req.accessories.unwrap_or_else(|| d["accessories"].as_str().unwrap_or_default().to_string())),
        "doc_fatura": req.doc_fatura.unwrap_or_else(|| d["doc_fatura"].as_str().unwrap_or_default().to_string()),
        "doc_garanti": req.doc_garanti.unwrap_or_else(|| d["doc_garanti"].as_str().unwrap_or_default().to_string()),
        "doc_uretim": req.doc_uretim.unwrap_or_else(|| d["doc_uretim"].as_str().unwrap_or_default().to_string()),
        "doc_ariza": req.doc_ariza.unwrap_or_else(|| d["doc_ariza"].as_str().unwrap_or_default().to_string()),
        "status": new_status,
    });

    document_store::update_document(&state.db.pool, COLLECTION, id, &data)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Statü değiştiyse SMS gönder
    if status_changed && state.config.sms.enabled {
        let phone = data["phone"].as_str().unwrap_or_default().to_string();
        let name = data["customer_name"].as_str().unwrap_or_default().to_string();
        let brand = data["brand"].as_str().unwrap_or_default().to_string();
        let model = data["model"].as_str().unwrap_or_default().to_string();
        let brand_model = format!("{} {}", brand, model);
        let sn = data["serial_number"].as_str().unwrap_or_default().to_string();
        let sms_status = new_status.to_string();
        let sms_base = state.config.sms.base_url.clone();
        let sms_user = state.config.sms.username.clone();
        let sms_pass = state.config.sms.password.clone();
        let sms_sender = state.config.sms.sender.clone();
        let pool = state.db.pool.clone();
        let record_id = id.to_string();
        tokio::spawn(async move {
            let cfg = crate::config::SmsConfig {
                base_url: sms_base,
                username: sms_user,
                password: sms_pass,
                sender: sms_sender,
                enabled: true,
            };
            crate::services::sms_service::send_status_sms(&cfg, &pool, &record_id, &phone, &name, &brand_model, &sn, &sms_status).await;
        });
    }

    let doc = document_store::find_by_id(&state.db.pool, COLLECTION, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::Internal("Güncelleme sonrası okuma hatası".into()))?;

    Ok(doc_to_response(&doc))
}

pub async fn delete_service_record(
    state: &Arc<AppState>,
    id: &str,
) -> Result<(), AppError> {
    let deleted = document_store::delete_document(&state.db.pool, COLLECTION, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !deleted {
        return Err(AppError::NotFound(format!("Kayıt {} bulunamadı", id)));
    }
    Ok(())
}

fn doc_to_response(doc: &document_store::Document) -> ServiceRecordResponse {
    let d = &doc.data;
    ServiceRecordResponse {
        id: doc.id.clone(),
        customer_name: d["customer_name"].as_str().unwrap_or_default().to_string(),
        phone: d["phone"].as_str().unwrap_or_default().to_string(),
        device: d["device"].as_str().unwrap_or_default().to_string(),
        brand: d["brand"].as_str().unwrap_or_default().to_string(),
        model: d["model"].as_str().unwrap_or_default().to_string(),
        serial_number: d["serial_number"].as_str().unwrap_or_default().to_string(),
        issue: d["issue"].as_str().unwrap_or_default().to_string(),
        notes: d["notes"].as_str().unwrap_or_default().to_string(),
        accessories: d["accessories"].as_str().unwrap_or_default().to_string(),
        doc_fatura: d["doc_fatura"].as_str().unwrap_or_default().to_string(),
        doc_garanti: d["doc_garanti"].as_str().unwrap_or_default().to_string(),
        doc_uretim: d["doc_uretim"].as_str().unwrap_or_default().to_string(),
        doc_ariza: d["doc_ariza"].as_str().unwrap_or_default().to_string(),
        status: d["status"].as_str().unwrap_or_default().to_string(),
        created_at: doc.created_at.clone(),
        updated_at: doc.updated_at.clone(),
    }
}
