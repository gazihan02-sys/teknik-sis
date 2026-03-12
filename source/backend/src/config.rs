pub struct SmsConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub sender: String,
    pub enabled: bool,
}

pub struct AppConfig {
    pub database_url: String,
    pub backend_host: String,
    pub backend_port: u16,
    pub sms: SmsConfig,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://teknik.db".to_string()),
            backend_host: std::env::var("BACKEND_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            backend_port: std::env::var("BACKEND_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            sms: SmsConfig {
                base_url: std::env::var("SMS_BASE_URL")
                    .unwrap_or_else(|_| "http://smsvt.voicetelekom.com:9587".to_string()),
                username: std::env::var("SMS_USERNAME")
                    .unwrap_or_else(|_| "rifaterdinc".to_string()),
                password: std::env::var("SMS_PASSWORD")
                    .unwrap_or_else(|_| "KgVaD5Gr".to_string()),
                sender: std::env::var("SMS_SENDER")
                    .unwrap_or_else(|_| "RIFATERDINC".to_string()),
                enabled: std::env::var("SMS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        }
    }
}
