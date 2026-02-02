use axum::Json;
use axum::routing::{get as axum_get, MethodRouter};

use super::{levels::levels_config, locales::locales_config};

pub fn get_locales() -> MethodRouter<crate::routes::api::ApiState> {
    axum_get(|| async { Json(locales_config()) })
}

pub fn get_levels() -> MethodRouter<crate::routes::api::ApiState> {
    axum_get(|| async { Json(levels_config()) })
}
