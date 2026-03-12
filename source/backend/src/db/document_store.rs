use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::FixedOffset;

/// Türkiye saat dilimi (UTC+3)
fn now_turkey() -> chrono::NaiveDateTime {
    let offset = FixedOffset::east_opt(3 * 3600).unwrap();
    chrono::Utc::now().with_timezone(&offset).naive_local()
}

/// Esnek JSON belge deposu — SQLite üzerinde (teknik.db).
/// Her belge bir "koleksiyon" adı altında JSON olarak saklanır.
/// Şema gerektirmez; istediğiniz yapıda veri ekleyip sorgulayabilirsiniz.

// -- Temel CRUD işlemleri --

/// Koleksiyona yeni bir JSON belgesi ekler. ID otomatik üretilir.
pub async fn insert_document(
    pool: &SqlitePool,
    collection: &str,
    data: &Value,
) -> Result<String, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let data_str = data.to_string();
    let now = now_turkey();

    sqlx::query(
        "INSERT INTO documents (id, collection, data, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(collection)
    .bind(&data_str)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Belirli bir koleksiyondaki tüm belgeleri getirir.
pub async fn find_all(
    pool: &SqlitePool,
    collection: &str,
) -> Result<Vec<Document>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, data, created_at, updated_at FROM documents WHERE collection = ? ORDER BY created_at DESC"
    )
    .bind(collection)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id, data, created_at, updated_at)| Document {
        id,
        data: serde_json::from_str(&data).unwrap_or(Value::Null),
        created_at,
        updated_at,
    }).collect())
}

/// ID'ye göre tek bir belge getirir.
pub async fn find_by_id(
    pool: &SqlitePool,
    collection: &str,
    id: &str,
) -> Result<Option<Document>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, data, created_at, updated_at FROM documents WHERE collection = ? AND id = ?"
    )
    .bind(collection)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, data, created_at, updated_at)| Document {
        id,
        data: serde_json::from_str(&data).unwrap_or(Value::Null),
        created_at,
        updated_at,
    }))
}

/// JSON alanı içinde arama yapar (SQLite json_extract ile).  
/// Örn: `find_by_field(pool, "products", "category", "elektronik")`
#[allow(dead_code)]
pub async fn find_by_field(
    pool: &SqlitePool,
    collection: &str,
    field: &str,
    value: &str,
) -> Result<Vec<Document>, sqlx::Error> {
    let json_path = format!("$.{}", field);
    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, data, created_at, updated_at FROM documents WHERE collection = ? AND json_extract(data, ?) = ? ORDER BY created_at DESC"
    )
    .bind(collection)
    .bind(&json_path)
    .bind(value)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id, data, created_at, updated_at)| Document {
        id,
        data: serde_json::from_str(&data).unwrap_or(Value::Null),
        created_at,
        updated_at,
    }).collect())
}

/// Belgeyi tamamen günceller (JSON'un tamamı değişir).
#[allow(dead_code)]
pub async fn update_document(
    pool: &SqlitePool,
    collection: &str,
    id: &str,
    data: &Value,
) -> Result<bool, sqlx::Error> {
    let data_str = data.to_string();
    let now = now_turkey();

    let result = sqlx::query(
        "UPDATE documents SET data = ?, updated_at = ? WHERE collection = ? AND id = ?"
    )
    .bind(&data_str)
    .bind(now)
    .bind(collection)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Belgeyi siler.
pub async fn delete_document(
    pool: &SqlitePool,
    collection: &str,
    id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM documents WHERE collection = ? AND id = ?"
    )
    .bind(collection)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Koleksiyondaki belge sayısını döndürür.
#[allow(dead_code)]
pub async fn count(
    pool: &SqlitePool,
    collection: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM documents WHERE collection = ?"
    )
    .bind(collection)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

// -- Veri yapıları --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub data: Value,
    pub created_at: String,
    pub updated_at: String,
}
