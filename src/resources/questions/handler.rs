use axum::routing::{get as axum_get, MethodRouter};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    resources::questions::{model::CreateQuestion, queries},
    routes::api::ApiState,
};

pub fn get() -> MethodRouter<ApiState> {
    axum_get(get_question)
}

pub fn collection() -> MethodRouter<ApiState> {
    axum_get(list_questions).post(create_question)
}

pub async fn get_question(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match queries::find_question_by_id(&state.db, &id).await {
        Ok(Some(q)) => (StatusCode::OK, Json(q)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "question not found" })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "failed to fetch question" })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub stage: Option<i32>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Serialize)]
pub struct QuestionsList {
    pub items: Vec<crate::resources::questions::model::QuestionDto>,
}

pub async fn list_questions(
    State(state): State<ApiState>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).min(100) as i64;
    let offset = params.offset.unwrap_or(0);

    match queries::list_questions(&state.db, params.stage, limit, offset).await {
        Ok(items) => (StatusCode::OK, Json(QuestionsList { items })).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "failed to list questions" })),
        )
            .into_response(),
    }
}

pub async fn delete_question(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match queries::delete_question_by_id(&state.db, &id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "question not found" })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "failed to delete question" })),
        )
            .into_response(),
    }
}

pub async fn create_question(
    State(state): State<ApiState>,
    Json(payload): Json<CreateQuestion>,
) -> impl IntoResponse {
    if let Err(msg) = validate_create_question(&payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": msg })),
        )
            .into_response();
    }

    match queries::insert_question(&state.db, payload).await {
        Ok(dto) => (StatusCode::CREATED, Json(dto)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "failed to create question" })),
        )
            .into_response(),
    }
}

fn validate_create_question(payload: &CreateQuestion) -> Result<(), &'static str> {
    if payload.prompt.trim().is_empty() {
        return Err("prompt is required");
    }
    if payload.options.len() != 4 {
        return Err("exactly four options are required");
    }

    let correct_count = payload
        .options
        .iter()
        .filter(|opt| opt.correct)
        .count();
    if correct_count != 1 {
        return Err("exactly one option must be marked correct");
    }

    if payload
        .options
        .iter()
        .any(|opt| opt.text.trim().is_empty())
    {
        return Err("all options must have non-empty text");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_create_question;
    use crate::resources::questions::model::{CreateQuestion, OptionItem};

    fn valid_question() -> CreateQuestion {
        CreateQuestion {
            stage: 1,
            prompt: "Prompt".to_string(),
            options: vec![
                OptionItem {
                    text: "A".to_string(),
                    correct: true,
                    explanation: None,
                },
                OptionItem {
                    text: "B".to_string(),
                    correct: false,
                    explanation: None,
                },
                OptionItem {
                    text: "C".to_string(),
                    correct: false,
                    explanation: None,
                },
                OptionItem {
                    text: "D".to_string(),
                    correct: false,
                    explanation: None,
                },
            ],
            tags: vec!["tag".to_string()],
        }
    }

    #[test]
    fn accepts_valid_question() {
        let q = valid_question();
        assert!(validate_create_question(&q).is_ok());
    }

    #[test]
    fn rejects_wrong_option_count() {
        let mut q = valid_question();
        q.options.pop();
        assert!(validate_create_question(&q).is_err());
    }

    #[test]
    fn rejects_empty_prompt() {
        let mut q = valid_question();
        q.prompt = "   ".to_string();
        assert!(validate_create_question(&q).is_err());
    }

    #[test]
    fn rejects_multiple_correct() {
        let mut q = valid_question();
        q.options[1].correct = true;
        assert!(validate_create_question(&q).is_err());
    }

    #[test]
    fn rejects_no_correct() {
        let mut q = valid_question();
        q.options.iter_mut().for_each(|o| o.correct = false);
        assert!(validate_create_question(&q).is_err());
    }

    #[test]
    fn rejects_empty_option_text() {
        let mut q = valid_question();
        q.options[0].text = "   ".to_string();
        assert!(validate_create_question(&q).is_err());
    }
}
