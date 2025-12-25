use mongodb::bson::{Bson, DateTime};
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuestion {
    pub stage: i32,
    pub prompt: String,
    pub options: Vec<OptionItem>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionItem {
    pub text: String,
    pub correct: bool,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    #[serde(rename = "_id")]
    pub id: Bson,
    pub stage: i32,
    pub prompt: String,
    pub options: Vec<OptionItem>,
    pub tags: Vec<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionDto {
    pub text: String,
    pub correct: bool,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionDto {
    pub id: String,
    pub stage: i32,
    pub prompt: String,
    pub options: Vec<OptionDto>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Question> for QuestionDto {
    fn from(q: Question) -> Self {
        Self {
            id: match q.id {
                Bson::ObjectId(oid) => oid.to_hex(),
                Bson::String(s) => s,
                other => other.to_string(),
            },
            stage: q.stage,
            prompt: q.prompt,
            options: q.options.into_iter().map(OptionDto::from).collect(),
            tags: q.tags,
            created_at: q.created_at.to_chrono().with_timezone(&Utc).to_rfc3339(),
            updated_at: q.updated_at.to_chrono().with_timezone(&Utc).to_rfc3339(),
        }
    }
}

impl From<OptionItem> for OptionDto {
    fn from(o: OptionItem) -> Self {
        Self {
            text: o.text,
            correct: o.correct,
            explanation: o.explanation,
        }
    }
}
