use axum::routing::{get as axum_get, MethodRouter};
use axum::{http::StatusCode, Json, extract::State};
use mongodb::bson::doc;
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

pub fn get_db() -> MethodRouter<crate::routes::api::ApiState> {
    axum_get(health_db)
}

#[derive(Serialize)]
pub struct HealthDbResponse {
    status: &'static str,
}

pub async fn health_db(
    State(state): State<crate::routes::api::ApiState>,
) -> (StatusCode, Json<HealthDbResponse>) {
    match state.db.run_command(doc! { "ping": 1 }, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthDbResponse { status: "ok" }),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthDbResponse { status: "degraded" }),
        ),
    }
}
