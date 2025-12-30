use serde_json::json;

pub fn levels_config() -> serde_json::Value {
    json!({
        "levels": [
            { "id": "lay", "label": { "en": "Lay Faithful", "es": "Laico", "pt": "Leigo" } },
            { "id": "convert", "label": { "en": "Convert", "es": "Converso", "pt": "Convertido" } },
            { "id": "religious", "label": { "en": "Religious", "es": "Religioso", "pt": "Religioso" } },
            { "id": "brother", "label": { "en": "Brother", "es": "Hermano", "pt": "Irmão" } },
            { "id": "sister", "label": { "en": "Sister", "es": "Hermana", "pt": "Irmã" } },
            { "id": "monk", "label": { "en": "Monk", "es": "Monje", "pt": "Monge" } },
            { "id": "priest", "label": { "en": "Priest", "es": "Sacerdote", "pt": "Sacerdote" } },
            { "id": "teacher", "label": { "en": "Teacher", "es": "Maestro", "pt": "Mestre" } },
            { "id": "pastor", "label": { "en": "Pastor", "es": "Pastor", "pt": "Pastor" } },
            { "id": "philosopher", "label": { "en": "Philosopher", "es": "Filósofo", "pt": "Filósofo" } },
            { "id": "doctor", "label": { "en": "Doctor of the Church", "es": "Doctor de la Iglesia", "pt": "Doutor da Igreja" } },
            { "id": "saint", "label": { "en": "Saint", "es": "Santo", "pt": "Santo" } }
        ]
    })
}
