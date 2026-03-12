use shared::dto::user_dto::{CreateUserRequest, UserResponse};
use shared::dto::product_dto::{CreateProductRequest, ProductResponse};
use shared::dto::auth_dto::{LoginRequest, LoginResponse, MeResponse};
use gloo_net::http::Request;

const API_BASE: &str = "/api";

// ---- Users ----

pub async fn get_users() -> Result<Vec<UserResponse>, String> {
    let resp = Request::get(&format!("{}/users", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

pub async fn create_user(req: CreateUserRequest) -> Result<UserResponse, String> {
    let resp = Request::post(&format!("{}/users", API_BASE))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

pub async fn delete_user(id: &str) -> Result<(), String> {
    let resp = Request::delete(&format!("{}/users/{}", API_BASE, id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

// ---- Products ----

pub async fn get_products() -> Result<Vec<ProductResponse>, String> {
    let resp = Request::get(&format!("{}/products", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

pub async fn create_product(req: CreateProductRequest) -> Result<ProductResponse, String> {
    let resp = Request::post(&format!("{}/products", API_BASE))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

pub async fn delete_product(id: &str) -> Result<(), String> {
    let resp = Request::delete(&format!("{}/products/{}", API_BASE, id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

// ---- Service Records ----

use shared::dto::service_record_dto::{
    CreateServiceRecordRequest, ServiceRecordResponse, StatusCountResponse, UpdateServiceRecordRequest,
};

pub async fn get_service_records(status: Option<&str>) -> Result<Vec<ServiceRecordResponse>, String> {
    let url = match status {
        Some(s) => format!("{}/service-records?status={}", API_BASE, s),
        None => format!("{}/service-records", API_BASE),
    };
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

#[allow(dead_code)]
pub async fn get_status_counts() -> Result<Vec<StatusCountResponse>, String> {
    let resp = Request::get(&format!("{}/service-records/counts", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

pub async fn create_service_record(req: CreateServiceRecordRequest) -> Result<ServiceRecordResponse, String> {
    let resp = Request::post(&format!("{}/service-records", API_BASE))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

#[allow(dead_code)]
pub async fn update_service_record(id: &str, req: UpdateServiceRecordRequest) -> Result<ServiceRecordResponse, String> {
    let resp = Request::put(&format!("{}/service-records/{}", API_BASE, id))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

pub async fn delete_service_record(id: &str) -> Result<(), String> {
    let resp = Request::delete(&format!("{}/service-records/{}", API_BASE, id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

// ---- SMS Log ----

use shared::dto::sms_log_dto::SmsLogResponse;

pub async fn get_sms_logs() -> Result<Vec<SmsLogResponse>, String> {
    let resp = Request::get(&format!("{}/sms-log", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

// ---- Auth ----

pub async fn login(username: &str, password: &str) -> Result<LoginResponse, String> {
    let req = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };
    let resp = Request::post(&format!("{}/auth/login", API_BASE))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err("Kullanıcı adı veya şifre hatalı".to_string())
    }
}

pub async fn check_auth(token: &str) -> Result<MeResponse, String> {
    let resp = Request::post(&format!("{}/auth/me", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err("Unauthorized".to_string())
    }
}

pub async fn logout_api(token: &str) -> Result<(), String> {
    let resp = Request::post(&format!("{}/auth/logout", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}
