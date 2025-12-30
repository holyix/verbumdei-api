use axum::routing::{MethodRouter, get as axum_get};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;

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

pub async fn get_question(State(state): State<ApiState>, Path(id): Path<String>) -> impl IntoResponse {
    match queries::find_question_by_id(&state.db, &id).await {
        Ok(Some(q)) => (StatusCode::OK, Json(q)).into_response(),
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "question not found" }))).into_response()
        }
        Err(err) => {
            error!(error = ?err, "failed to fetch question");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "failed to fetch question" })))
                .into_response()
        }
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
        Ok(items) => (
            StatusCode::OK,
            Json(QuestionsList {
                items,
            }),
        )
            .into_response(),
        Err(err) => {
            error!(error = ?err, "failed to list questions");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "failed to list questions" })))
                .into_response()
        }
    }
}

pub async fn delete_question(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match queries::delete_question_by_id(&state.db, &id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "question not found" }))).into_response()
        }
        Err(err) => {
            error!(error = ?err, "failed to delete question");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "failed to delete question" })))
                .into_response()
        }
    }
}

pub async fn create_question(
    State(state): State<ApiState>,
    Json(payload): Json<CreateQuestion>,
) -> impl IntoResponse {
    if let Err(msg) = validate_create_question(&payload) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response();
    }

    match queries::insert_question(&state.db, payload).await {
        Ok(dto) => (StatusCode::CREATED, Json(dto)).into_response(),
        Err(err) => {
            error!(error = ?err, "failed to create question");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "failed to create question" })))
                .into_response()
        }
    }
}

fn validate_create_question(payload: &CreateQuestion) -> Result<(), &'static str> {
    validate_localized(&payload.prompt, "prompt")?;
    if let Some(label) = &payload.stage_label {
        validate_localized(label, "stage_label")?;
    }

    if payload.options.len() != 4 {
        return Err("exactly four options are required");
    }

    let correct_count = payload.options.iter().filter(|opt| opt.correct).count();
    if correct_count != 1 {
        return Err("exactly one option must be marked correct");
    }

    for opt in &payload.options {
        validate_localized(&opt.text, "option text")?;
        if let Some(expl) = &opt.explanation {
            validate_localized(expl, "option explanation")?;
        }
    }

    Ok(())
}

fn validate_localized(
    text: &crate::resources::questions::model::LocalizedText,
    field: &'static str,
) -> Result<(), &'static str> {
    const REQUIRED: [&str; 3] = ["en", "es", "pt"];
    for locale in REQUIRED {
        match text.get(locale) {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                return Err(match field {
                    "prompt" => "prompt requires en, es, and pt",
                    "stage_label" => "stage_label requires en, es, and pt",
                    "option text" => "option text requires en, es, and pt",
                    "option explanation" => "option explanation requires en, es, and pt",
                    _ => "all locales must be provided",
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_create_question, validate_localized};
    use crate::resources::questions::model::{CreateQuestion, LocalizedText, OptionItem};

    fn valid_question() -> CreateQuestion {
        let mut stage_label = LocalizedText::new();
        stage_label.insert("en".into(), "Stage".into());
        stage_label.insert("es".into(), "Etapa".into());
        stage_label.insert("pt".into(), "Etapa".into());

        let mut prompt = LocalizedText::new();
        prompt.insert("en".into(), "Prompt".into());
        prompt.insert("es".into(), "Pregunta".into());
        prompt.insert("pt".into(), "Pergunta".into());

        let mut opt_a = LocalizedText::new();
        opt_a.insert("en".into(), "A".into());
        opt_a.insert("es".into(), "A".into());
        opt_a.insert("pt".into(), "A".into());

        let mut opt_b = LocalizedText::new();
        opt_b.insert("en".into(), "B".into());
        opt_b.insert("es".into(), "B".into());
        opt_b.insert("pt".into(), "B".into());

        let mut opt_c = LocalizedText::new();
        opt_c.insert("en".into(), "C".into());
        opt_c.insert("es".into(), "C".into());
        opt_c.insert("pt".into(), "C".into());

        let mut opt_d = LocalizedText::new();
        opt_d.insert("en".into(), "D".into());
        opt_d.insert("es".into(), "D".into());
        opt_d.insert("pt".into(), "D".into());

        let mut expl_b = LocalizedText::new();
        expl_b.insert("en".into(), "because".into());
        expl_b.insert("es".into(), "porque".into());
        expl_b.insert("pt".into(), "porque".into());

        CreateQuestion {
            stage: 1,
            stage_label: Some(stage_label),
            prompt,
            options: vec![
                OptionItem {
                    text: opt_a,
                    correct: true,
                    explanation: None,
                },
                OptionItem {
                    text: opt_b,
                    correct: false,
                    explanation: Some(expl_b),
                },
                OptionItem {
                    text: opt_c,
                    correct: false,
                    explanation: None,
                },
                OptionItem {
                    text: opt_d,
                    correct: false,
                    explanation: None,
                },
            ],
            tags: vec!["tag".to_string()],
            image_url: None,
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
        q.prompt.insert("en".into(), "   ".into());
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
        q.options[0].text.insert("en".into(), "   ".into());
        assert!(validate_create_question(&q).is_err());
    }

    #[test]
    fn validate_localized_requires_all_locales() {
        let mut lt = LocalizedText::new();
        lt.insert("en".into(), "a".into());
        lt.insert("es".into(), "b".into());
        lt.insert("pt".into(), "c".into());
        assert!(validate_localized(&lt, "prompt").is_ok());
        lt.insert("es".into(), "".into());
        assert!(validate_localized(&lt, "prompt").is_err());
    }
}
