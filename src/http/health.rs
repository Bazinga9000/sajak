use actix_web::HttpResponse;
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct HealthCheck {
    status: String,
    version: String,
}

// Health check endpoint
pub async fn health() -> HttpResponse {
    let response = HealthCheck {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    HttpResponse::Ok().json(response)
}
