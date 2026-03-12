use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRecordRequest {
    pub customer_name: String,
    pub phone: String,
    pub device: String,
    pub brand: String,
    pub model: String,
    pub serial_number: String,
    pub issue: String,
    pub notes: String,
    #[serde(default)]
    pub accessories: String,
    #[serde(default)]
    pub doc_fatura: String,
    #[serde(default)]
    pub doc_garanti: String,
    #[serde(default)]
    pub doc_uretim: String,
    #[serde(default)]
    pub doc_ariza: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceRecordRequest {
    pub customer_name: Option<String>,
    pub phone: Option<String>,
    pub device: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub issue: Option<String>,
    pub notes: Option<String>,
    pub accessories: Option<String>,
    pub status: Option<String>,
    pub doc_fatura: Option<String>,
    pub doc_garanti: Option<String>,
    pub doc_uretim: Option<String>,
    pub doc_ariza: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecordResponse {
    pub id: String,
    pub customer_name: String,
    pub phone: String,
    pub device: String,
    pub brand: String,
    pub model: String,
    pub serial_number: String,
    pub issue: String,
    pub notes: String,
    #[serde(default)]
    pub accessories: String,
    #[serde(default)]
    pub doc_fatura: String,
    #[serde(default)]
    pub doc_garanti: String,
    #[serde(default)]
    pub doc_uretim: String,
    #[serde(default)]
    pub doc_ariza: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCountResponse {
    pub status: String,
    pub count: i64,
}
