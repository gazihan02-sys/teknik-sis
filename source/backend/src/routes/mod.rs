pub mod auth_routes;
pub mod user_routes;
pub mod product_routes;
pub mod service_record_routes;
pub mod sms_routes;
pub mod health;

use std::sync::Arc;
use axum::Router;
use crate::AppState;

pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::routes())
        .nest("/auth", auth_routes::routes())
        .nest("/users", user_routes::routes())
        .nest("/products", product_routes::routes())
        .nest("/service-records", service_record_routes::routes())
        .nest("/sms-log", sms_routes::routes())
}
