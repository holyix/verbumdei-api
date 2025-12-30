use serde_json::json;

pub fn locales_config() -> serde_json::Value {
    json!({
        "languages": [
            { "id": "en", "label": "EN", "name": "English", "flag": "🇬🇧" },
            { "id": "es", "label": "ES", "name": "Español", "flag": "🇪🇸" },
            { "id": "pt", "label": "PT", "name": "Português", "flag": "🇧🇷" }
        ]
    })
}
