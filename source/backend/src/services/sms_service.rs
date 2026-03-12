use crate::config::SmsConfig;
use sqlx::SqlitePool;

const SMS_LOG_COLLECTION: &str = "sms_log";

/// Aynı kayıt + statü için daha önce SMS gönderilmiş mi kontrol eder.
async fn already_sent(pool: &SqlitePool, record_id: &str, status: &str) -> bool {
    let key = format!("{}_{}", record_id, status);
    let docs = crate::db::document_store::find_by_field(pool, SMS_LOG_COLLECTION, "key", &key)
        .await
        .unwrap_or_default();
    !docs.is_empty()
}

/// SMS gönderim kaydını veritabanına yazar.
async fn log_sms(pool: &SqlitePool, record_id: &str, status: &str, phone: &str, customer_name: &str, content: &str) {
    let key = format!("{}_{}", record_id, status);
    let data = serde_json::json!({
        "key": key,
        "record_id": record_id,
        "status": status,
        "phone": phone,
        "customer_name": customer_name,
        "content": content,
    });
    let _ = crate::db::document_store::insert_document(pool, SMS_LOG_COLLECTION, &data).await;
}

/// Tüm SMS loglarını getirir.
pub async fn list_sms_logs(pool: &SqlitePool) -> Vec<crate::db::document_store::Document> {
    crate::db::document_store::find_all(pool, SMS_LOG_COLLECTION)
        .await
        .unwrap_or_default()
}

/// Statü anahtarından SMS mesaj içeriği üretir.
/// Müşteri adı ve marka/model bilgisi mesaja eklenir.
fn sms_content_for_status(status: &str, customer_name: &str, brand_model: &str, serial_number: &str) -> Option<String> {
    let msg = match status {
        "musteri_kabul" => format!(
            "SN: {}, {} Cihazinin servisimize incelenmek uzere alinmistir. Teknisyenlerimiz en kisa surede incelemeye baslayacaktir. Fatura Yuklemek Icin: https://tamir.sis-teknik.com.tr 04162161262 - Teknik Elektronik",
            customer_name, brand_model
        ),
        "teknisyene_verildi" => format!(
            "Sayın {}, {} cihazınız teknisyene teslim edilmiştir. SİS TEKNİK",
            customer_name, brand_model
        ),
        "islem_bekliyor" => format!(
            "Sayın {}, {} cihazınız işlem beklemektedir. SİS TEKNİK",
            customer_name, brand_model
        ),
        "parca_bekliyor" => format!(
            "Sayın {}, {} cihazınız için parça beklenmektedir. SİS TEKNİK",
            customer_name, brand_model
        ),
        "merkeze_sevk" => format!(
            "Sayın {}, {} cihazınız merkeze sevk edilmiştir. SİS TEKNİK",
            customer_name, brand_model
        ),
        "degisim" => format!(
            "Sayın {}, {} cihazınız değişim sürecine alınmıştır. SİS TEKNİK",
            customer_name, brand_model
        ),
        "tamir_tamamlandi" => format!(
            "Sayın {}, {} cihazınızın tamiri tamamlanmıştır. Teslim almak için servisimize bekleriz. SİS TEKNİK",
            customer_name, brand_model
        ),
        "teslim_edildi" => format!(
            "Sayın {}, {} cihazınız teslim edilmiştir. Bizi tercih ettiğiniz için teşekkür ederiz. SİS TEKNİK",
            customer_name, brand_model
        ),
        "iade" => format!(
            "Sayın {}, {} cihazınız iade sürecine alınmıştır. SİS TEKNİK",
            customer_name, brand_model
        ),
        _ => return None,
    };
    Some(msg)
}

/// Statü değişikliğinde SMS gönderir. Aynı kayıt+statü için tekrar göndermez.
pub async fn send_status_sms(
    sms_config: &SmsConfig,
    pool: &SqlitePool,
    record_id: &str,
    phone: &str,
    customer_name: &str,
    brand_model: &str,
    serial_number: &str,
    status: &str,
) {
    let phone = phone.trim();
    if phone.is_empty() {
        tracing::warn!("SMS atlanıyor: telefon numarası boş");
        return;
    }

    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 10 {
        tracing::warn!("SMS atlanıyor: geçersiz telefon numarası '{}'", phone);
        return;
    }

    // Daha önce aynı kayıt+statü için SMS gittiyse tekrar gönderme
    if already_sent(pool, record_id, status).await {
        tracing::info!("SMS atlanıyor: {} kaydı için '{}' statüsünde zaten gönderilmiş", record_id, status);
        return;
    }

    let content = match sms_content_for_status(status, customer_name, brand_model, serial_number) {
        Some(c) => c,
        None => {
            tracing::warn!("SMS atlanıyor: bilinmeyen statü '{}'", status);
            return;
        }
    };

    let body = serde_json::json!({
        "type": 1,
        "sendingType": 0,
        "title": format!("Servis - {}", status),
        "content": content,
        "number": digits,
        "encoding": 1,
        "sender": sms_config.sender,
        "periodicSettings": null,
        "sendingDate": null,
        "validity": 60,
        "pushSettings": null
    });

    let client = reqwest::Client::new();
    let result = client
        .post(&format!("{}/sms/create", sms_config.base_url))
        .basic_auth(&sms_config.username, Some(&sms_config.password))
        .json(&body)
        .send()
        .await;

    match result {
        Ok(resp) => {
            let status_code = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if status_code.is_success() {
                tracing::info!("SMS gönderildi [{}]: {} -> {}", status, digits, text);
                // Başarılı gönderimi logla
                log_sms(pool, record_id, status, &digits, customer_name, &content).await;
            } else {
                tracing::error!("SMS hata [{}] ({}): {}", status, status_code, text);
            }
        }
        Err(e) => {
            tracing::error!("SMS gönderim hatası [{}]: {}", status, e);
        }
    }
}
