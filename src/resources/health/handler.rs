use axum::routing::{MethodRouter, get as axum_get};
use axum::{Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub fn get() -> MethodRouter<crate::routes::api::ApiState> {
    axum_get(health_check)
}

pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
        }),
    )
}
