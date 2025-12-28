use axum::{Json, Router, http::{Method, StatusCode}, response::IntoResponse};
use mongodb::Database;
use serde_json::json;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

use crate::resources::{health::handler as health_handler, questions::handler as question_handler};

#[derive(Clone)]
pub struct ApiState {
    pub db: Database,
}

pub fn router(state: ApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any);

    Router::new()
        .route("/health", health_handler::get())
        .route("/health/db", health_handler::get_db())
        .route("/v1/questions/:id", question_handler::get().delete(question_handler::delete_question))
        .route("/v1/questions", question_handler::collection())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .fallback(fallback_invalid_path)
        .with_state(state)
}

async fn fallback_invalid_path() -> impl IntoResponse {
    (StatusCode::CONFLICT, Json(json!({ "error": "invalid path" })))
}
