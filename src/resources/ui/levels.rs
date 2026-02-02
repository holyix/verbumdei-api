use serde_json::json;

pub fn levels_config() -> serde_json::Value {
    json!({
        "levels": [
            { "id": "lay", "label": { "en": "Lay Faithful", "es": "Laico", "pt": "Leigo", "sv": "Lekman" } },
            { "id": "convert", "label": { "en": "Convert", "es": "Converso", "pt": "Convertido", "sv": "Konvertit" } },
            { "id": "religious", "label": { "en": "Religious", "es": "Religioso", "pt": "Religioso", "sv": "Religiös" } },
            { "id": "brother", "label": { "en": "Brother", "es": "Hermano", "pt": "Irmão", "sv": "Broder" } },
            { "id": "sister", "label": { "en": "Sister", "es": "Hermana", "pt": "Irmã", "sv": "Syster" } },
            { "id": "monk", "label": { "en": "Monk", "es": "Monje", "pt": "Monge", "sv": "Munk" } },
            { "id": "priest", "label": { "en": "Priest", "es": "Sacerdote", "pt": "Sacerdote", "sv": "Präst" } },
            { "id": "teacher", "label": { "en": "Teacher", "es": "Maestro", "pt": "Mestre", "sv": "Lärare" } },
            { "id": "pastor", "label": { "en": "Pastor", "es": "Pastor", "pt": "Pastor", "sv": "Pastor" } },
            { "id": "philosopher", "label": { "en": "Philosopher", "es": "Filósofo", "pt": "Filósofo", "sv": "Filosof" } },
            { "id": "doctor", "label": { "en": "Doctor of the Church", "es": "Doctor de la Iglesia", "pt": "Doutor da Igreja", "sv": "Kyrkolärare" } },
            { "id": "saint", "label": { "en": "Saint", "es": "Santo", "pt": "Santo", "sv": "Helgon" } }
        ]
    })
}
