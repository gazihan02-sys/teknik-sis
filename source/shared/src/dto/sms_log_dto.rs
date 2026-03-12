use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmsLogResponse {
    pub id: String,
    pub record_id: String,
    pub status: String,
    pub phone: String,
    pub customer_name: String,
    pub content: String,
    pub created_at: String,
}
