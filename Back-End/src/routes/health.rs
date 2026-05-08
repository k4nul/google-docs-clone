use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub timestamp: DateTime<Utc>,
}

pub async fn get_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "backend",
        timestamp: Utc::now(),
    })
}
