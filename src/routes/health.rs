use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse { status: "ok" }),
    )
}
