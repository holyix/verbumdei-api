use axum::routing::{MethodRouter, get as axum_get};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    resources::eras::queries::{self, EpisodeLookup},
    routes::api::ApiState,
};

#[derive(Serialize)]
struct ErrorEnvelope {
    error: &'static str,
    message: String,
}

#[derive(Deserialize)]
pub struct EpisodesSearchQuery {
    pub book: Option<String>,
    pub lang: Option<String>,
}

#[derive(Deserialize)]
pub struct LangQuery {
    pub lang: Option<String>,
}

pub fn collection() -> MethodRouter<ApiState> {
    axum_get(list_eras)
}

pub fn era() -> MethodRouter<ApiState> {
    axum_get(get_era)
}

pub fn episodes_collection() -> MethodRouter<ApiState> {
    axum_get(list_episodes_for_era)
}

pub fn episode() -> MethodRouter<ApiState> {
    axum_get(get_episode)
}

pub fn episodes_search() -> MethodRouter<ApiState> {
    axum_get(search_episodes)
}

pub async fn list_eras(
    State(state): State<ApiState>,
    Query(params): Query<LangQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let lang = resolve_lang(params.lang.as_deref(), headers.get("accept-language"));
    match queries::list_eras(&state.db, &lang).await {
        Ok(eras) => (StatusCode::OK, Json(eras)).into_response(),
        Err(err) => {
            error!(error = ?err, "failed to list eras");
            internal_error_response("failed to list eras")
        }
    }
}

pub async fn get_era(
    State(state): State<ApiState>,
    Path(era_id): Path<String>,
    Query(params): Query<LangQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if era_id.trim().is_empty() {
        return bad_request_response("eraId must not be empty");
    }
    let lang = resolve_lang(params.lang.as_deref(), headers.get("accept-language"));

    match queries::find_era_by_id(&state.db, &era_id, &lang).await {
        Ok(Some(era)) => (StatusCode::OK, Json(era)).into_response(),
        Ok(None) => not_found_response("Era not found"),
        Err(err) => {
            error!(error = ?err, "failed to fetch era");
            internal_error_response("failed to fetch era")
        }
    }
}

pub async fn list_episodes_for_era(
    State(state): State<ApiState>,
    Path(era_id): Path<String>,
    Query(params): Query<LangQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if era_id.trim().is_empty() {
        return bad_request_response("eraId must not be empty");
    }
    let lang = resolve_lang(params.lang.as_deref(), headers.get("accept-language"));

    match queries::list_episodes_for_era(&state.db, &era_id, &lang).await {
        Ok(Some(episodes)) => (StatusCode::OK, Json(episodes)).into_response(),
        Ok(None) => not_found_response("Era not found"),
        Err(err) => {
            error!(error = ?err, "failed to list episodes for era");
            internal_error_response("failed to list episodes for era")
        }
    }
}

pub async fn get_episode(
    State(state): State<ApiState>,
    Path((era_id, episode_id)): Path<(String, String)>,
    Query(params): Query<LangQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if era_id.trim().is_empty() {
        return bad_request_response("eraId must not be empty");
    }
    if episode_id.trim().is_empty() {
        return bad_request_response("episodeId must not be empty");
    }
    let lang = resolve_lang(params.lang.as_deref(), headers.get("accept-language"));

    match queries::find_episode_for_era(&state.db, &era_id, &episode_id, &lang).await {
        Ok(EpisodeLookup::Found(episode)) => (StatusCode::OK, Json(episode)).into_response(),
        Ok(EpisodeLookup::EraNotFound) => not_found_response("Era not found"),
        Ok(EpisodeLookup::EpisodeNotFound) => not_found_response("Episode not found under era"),
        Err(err) => {
            error!(error = ?err, "failed to fetch episode");
            internal_error_response("failed to fetch episode")
        }
    }
}

pub async fn search_episodes(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(params): Query<EpisodesSearchQuery>,
) -> impl IntoResponse {
    let lang = resolve_lang(params.lang.as_deref(), headers.get("accept-language"));
    let Some(book) = params.book else {
        return bad_request_response("book query parameter is required");
    };

    let book = book.trim();
    if book.is_empty() {
        return bad_request_response("book query parameter must not be empty");
    }

    match queries::search_episodes_by_book(&state.db, book, &lang).await {
        Ok(episodes) => (StatusCode::OK, Json(episodes)).into_response(),
        Err(err) => {
            error!(error = ?err, "failed to search episodes by book");
            internal_error_response("failed to search episodes")
        }
    }
}

fn not_found_response(message: impl Into<String>) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorEnvelope {
            error: "NotFound",
            message: message.into(),
        }),
    )
        .into_response()
}

fn bad_request_response(message: impl Into<String>) -> axum::response::Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorEnvelope {
            error: "BadRequest",
            message: message.into(),
        }),
    )
        .into_response()
}

fn internal_error_response(message: impl Into<String>) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorEnvelope {
            error: "InternalError",
            message: message.into(),
        }),
    )
        .into_response()
}

fn resolve_lang(query_lang: Option<&str>, accept_language: Option<&axum::http::HeaderValue>) -> String {
    if let Some(lang) = query_lang.and_then(normalize_lang) {
        return lang.to_string();
    }

    if let Some(lang) =
        accept_language.and_then(|header| header.to_str().ok()).and_then(resolve_from_accept_language)
    {
        return lang.to_string();
    }

    "en".to_string()
}

fn resolve_from_accept_language(value: &str) -> Option<&str> {
    value
        .split(',')
        .filter_map(|part| {
            let base = part.trim().split(';').next()?.trim();
            normalize_lang(base)
        })
        .find(|lang| is_supported_lang(lang))
}

fn normalize_lang(value: &str) -> Option<&str> {
    let normalized = value.trim().split('-').next()?.to_ascii_lowercase();
    match normalized.as_str() {
        "en" => Some("en"),
        "es" => Some("es"),
        "pt" => Some("pt"),
        "sv" => Some("sv"),
        _ => None,
    }
}

fn is_supported_lang(value: &str) -> bool {
    matches!(value, "en" | "es" | "pt" | "sv")
}

#[cfg(test)]
mod tests {
    use super::resolve_from_accept_language;

    #[test]
    fn picks_first_supported_accept_language() {
        assert_eq!(resolve_from_accept_language("fr-FR, sv-SE;q=0.9, en;q=0.8"), Some("sv"));
    }
}
