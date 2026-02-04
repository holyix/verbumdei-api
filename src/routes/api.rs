use axum::{
    Json, Router,
    http::{Method, StatusCode},
    response::IntoResponse,
};
use mongodb::Database;
use serde_json::json;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::resources::{
    eras::handler as era_handler, health::handler as health_handler,
    questions::handler as question_handler, ui::handler as ui_handler,
};

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
        // Health routes
        .route("/health", health_handler::get())
        .route("/health/db", health_handler::get_db())
        // Config routes
        .route("/v1/ui/locales", ui_handler::get_locales())
        .route("/v1/ui/levels", ui_handler::get_levels())
        // Questions routes
        .route("/v1/questions/:id", question_handler::get().delete(question_handler::delete_question))
        .route("/v1/questions", question_handler::collection())
        // Eras routes
        .route("/eras", era_handler::collection())
        .route("/eras/:era_id", era_handler::era())
        .route("/eras/:era_id/episodes", era_handler::episodes_collection())
        .route("/eras/:era_id/episodes/:episode_id", era_handler::episode())
        .route("/episodes", era_handler::episodes_search())
        // Versioned aliases for eras routes
        .route("/v1/eras", era_handler::collection())
        .route("/v1/eras/:era_id", era_handler::era())
        .route("/v1/eras/:era_id/episodes", era_handler::episodes_collection())
        .route("/v1/eras/:era_id/episodes/:episode_id", era_handler::episode())
        .route("/v1/episodes", era_handler::episodes_search())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .fallback(fallback_invalid_path)
        .with_state(state)
}

async fn fallback_invalid_path() -> impl IntoResponse {
    (StatusCode::CONFLICT, Json(json!({ "error": "invalid path" })))
}
