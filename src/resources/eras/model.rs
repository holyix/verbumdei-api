use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Reference {
    pub book_id: String,
    pub book: String,
    pub chapters: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct EraListItem {
    pub id: String,
    pub name: String,
    pub label: String,
    pub image_path: Option<String>,
    pub order: i32,
    #[serde(rename = "type")]
    pub era_type: Option<String>,
    pub episode_count: usize,
}

#[derive(Debug, Serialize)]
pub struct EraDto {
    pub id: String,
    pub name: String,
    pub label: String,
    pub image_path: Option<String>,
    pub order: i32,
    #[serde(rename = "type")]
    pub era_type: Option<String>,
    pub books: Vec<String>,
    pub episodes: Vec<EpisodeDto>,
}

#[derive(Debug, Serialize)]
pub struct EpisodeListItem {
    pub id: String,
    pub name: String,
    pub label: String,
    pub order: i32,
    pub reference_count: usize,
}

#[derive(Debug, Serialize)]
pub struct EpisodeDto {
    pub id: String,
    pub name: String,
    pub label: String,
    pub order: i32,
    pub references: Vec<Reference>,
}

#[derive(Debug, Serialize)]
pub struct EpisodeSearchItem {
    pub era_id: String,
    pub era_label: String,
    pub id: String,
    pub label: String,
}
