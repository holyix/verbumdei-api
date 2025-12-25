use axum::{Json, Router, http::StatusCode, response::IntoResponse};
use mongodb::Database;
use serde_json::json;

use crate::resources::{health::handler as health_handler, questions::handler as question_handler};

#[derive(Clone)]
pub struct ApiState {
    pub db: Database,
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/health", health_handler::get())
        .route(
            "/v1/questions/:id",
            question_handler::get().delete(question_handler::delete_question),
        )
        .route("/v1/questions", question_handler::collection())
        .fallback(fallback_invalid_path)
        .with_state(state)
}

async fn fallback_invalid_path() -> impl IntoResponse {
    (StatusCode::CONFLICT, Json(json!({ "error": "invalid path" })))
}
