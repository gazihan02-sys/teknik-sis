pub mod sqlite;
pub mod document_store;

use crate::config::AppConfig;

pub struct Database {
    pub pool: sqlx::SqlitePool,
}

impl Database {
    pub async fn new(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let sqlite_opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(config.database_url.trim_start_matches("sqlite://"))
            .create_if_missing(true);

        let pool = sqlx::SqlitePool::connect_with(sqlite_opts).await?;
        tracing::info!("SQLite pool created");

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Geleneksel ilişkisel tablo: users
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL DEFAULT '',
                role TEXT NOT NULL DEFAULT 'user',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&self.pool)
        .await?;

        // password_hash sütunu yoksa ekle (mevcut DB'ler için)
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN password_hash TEXT NOT NULL DEFAULT ''")
            .execute(&self.pool)
            .await;

        // JSON belge deposu: esnek, şemasız koleksiyon tablosu
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                collection TEXT NOT NULL,
                data JSON NOT NULL DEFAULT '{}',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&self.pool)
        .await?;

        // Koleksiyon bazlı hızlı sorgulama için index
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection)"
        )
        .execute(&self.pool)
        .await?;

        // Varsayılan admin kullanıcısı yoksa oluştur
        self.seed_admin().await?;

        tracing::info!("SQLite migrations completed (relational + document store)");
        Ok(())
    }

    async fn seed_admin(&self) -> Result<(), Box<dyn std::error::Error>> {
        use sha2::{Sha256, Digest};

        // Admin kullanıcısı
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = 'admin'"
        )
        .fetch_one(&self.pool)
        .await?;

        if row == 0 {
            let mut hasher = Sha256::new();
            hasher.update(b"admin");
            let hash = format!("{:x}", hasher.finalize());

            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().naive_utc();
            sqlx::query(
                "INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&id)
            .bind("admin")
            .bind("admin@local")
            .bind(&hash)
            .bind("admin")
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            tracing::info!("Default admin user created (username: admin, password: admin)");
        }

        // Teknik kullanıcısı
        let row2 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = 'teknik'"
        )
        .fetch_one(&self.pool)
        .await?;

        if row2 == 0 {
            let mut hasher = Sha256::new();
            hasher.update(b"123456");
            let hash = format!("{:x}", hasher.finalize());

            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().naive_utc();
            sqlx::query(
                "INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&id)
            .bind("teknik")
            .bind("teknik@local")
            .bind(&hash)
            .bind("admin")
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            tracing::info!("Default teknik user created (username: teknik, password: 123456)");
        }

        Ok(())
    }
}
