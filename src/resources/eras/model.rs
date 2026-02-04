use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize)]
pub struct Era {
    #[serde(rename = "_id")]
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub era_type: Option<String>,
    #[serde(default)]
    pub books: Vec<String>,
    #[serde(default)]
    pub episodes: Vec<Episode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Episode {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub references: Vec<Reference>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Reference {
    pub book: String,
    #[serde(default)]
    pub chapters: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct EraListItem {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub era_type: Option<String>,
    pub episode_count: usize,
}

#[derive(Debug, Serialize)]
pub struct EraDto {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub era_type: Option<String>,
    pub books: Vec<String>,
    pub episodes: Vec<EpisodeDto>,
}

#[derive(Debug, Serialize)]
pub struct EpisodeListItem {
    pub id: String,
    pub label: String,
    pub reference_count: usize,
}

#[derive(Debug, Serialize)]
pub struct EpisodeDto {
    pub id: String,
    pub label: String,
    pub references: Vec<Reference>,
}

#[derive(Debug, Serialize)]
pub struct EpisodeSearchItem {
    pub era_id: String,
    pub era_label: String,
    pub id: String,
    pub label: String,
}

impl From<Era> for EraDto {
    fn from(value: Era) -> Self {
        Self {
            id: value.id,
            label: value.label,
            era_type: value.era_type,
            books: value.books,
            episodes: value.episodes.into_iter().map(EpisodeDto::from).collect(),
        }
    }
}

impl From<Era> for EraListItem {
    fn from(value: Era) -> Self {
        Self {
            id: value.id,
            label: value.label,
            era_type: value.era_type,
            episode_count: value.episodes.len(),
        }
    }
}

impl From<Episode> for EpisodeDto {
    fn from(value: Episode) -> Self {
        Self {
            id: value.id,
            label: value.label,
            references: value.references,
        }
    }
}

impl From<Episode> for EpisodeListItem {
    fn from(value: Episode) -> Self {
        Self {
            id: value.id,
            label: value.label,
            reference_count: value.references.len(),
        }
    }
}

#[cfg(test)]
pub fn validate_era_collection(eras: &[Era]) -> Result<(), String> {
    for era in eras {
        let mut ids = HashSet::new();
        for episode in &era.episodes {
            if !ids.insert(episode.id.as_str()) {
                return Err(format!("duplicate episode id '{}' in era '{}'", episode.id, era.id));
            }

            for reference in &episode.references {
                if era.books.iter().any(|book| book == &reference.book) {
                    continue;
                }

                if is_allowed_cross_era_bridge(&era.id, &episode.id, &reference.book) {
                    continue;
                }

                return Err(format!(
                    "reference book '{}' is not in era '{}' books",
                    reference.book, era.id
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
fn is_allowed_cross_era_bridge(era_id: &str, episode_id: &str, book: &str) -> bool {
    era_id == "gospel" && episode_id == "resurrection" && book == "Acts"
}

#[cfg(test)]
mod tests {
    use super::{Era, EraListItem, validate_era_collection};

    #[test]
    fn validates_well_formed_fixture() {
        let eras: Vec<Era> = serde_json::from_str(
            r#"[
                {
                    "_id": "creation",
                    "label": "Creation",
                    "books": ["Genesis"],
                    "episodes": [
                        { "id": "world", "label": "World", "references": [{ "book": "Genesis", "chapters": [1] }] }
                    ]
                }
            ]"#,
        )
        .expect("fixture should parse");
        assert!(validate_era_collection(&eras).is_ok());
    }

    #[test]
    fn rejects_duplicate_episode_ids() {
        let eras: Vec<Era> = serde_json::from_str(
            r#"[
                {
                    "_id": "e1",
                    "label": "Era",
                    "books": ["Genesis"],
                    "episodes": [
                        { "id": "dup", "label": "A", "references": [{"book": "Genesis", "chapters": [1]}] },
                        { "id": "dup", "label": "B", "references": [{"book": "Genesis", "chapters": [2]}] }
                    ]
                }
            ]"#,
        )
        .expect("fixture should parse");

        assert!(validate_era_collection(&eras).is_err());
    }

    #[test]
    fn rejects_unknown_books_without_bridge_rule() {
        let eras: Vec<Era> = serde_json::from_str(
            r#"[
                {
                    "_id": "e1",
                    "label": "Era",
                    "books": ["Genesis"],
                    "episodes": [
                        { "id": "ep", "label": "A", "references": [{"book": "Exodus", "chapters": [2]}] }
                    ]
                }
            ]"#,
        )
        .expect("fixture should parse");

        assert!(validate_era_collection(&eras).is_err());
    }

    #[test]
    fn allows_configured_cross_era_bridge_reference() {
        let eras: Vec<Era> = serde_json::from_str(
            r#"[
                {
                    "_id": "gospel",
                    "label": "Gospel",
                    "books": ["Matthew"],
                    "episodes": [
                        { "id": "resurrection", "label": "Resurrection", "references": [{"book": "Acts", "chapters": [1]}] }
                    ]
                }
            ]"#,
        )
        .expect("fixture should parse");

        assert!(validate_era_collection(&eras).is_ok());
    }

    #[test]
    fn era_list_item_counts_episodes() {
        let era: Era = serde_json::from_str(
            r#"{
                "_id": "creation",
                "label": "Creation",
                "books": ["Genesis"],
                "episodes": [
                    { "id": "world", "label": "World", "references": [] },
                    { "id": "humanity", "label": "Humanity", "references": [] }
                ]
            }"#,
        )
        .expect("fixture should parse");

        let list_item = EraListItem::from(era);
        assert_eq!(list_item.episode_count, 2);
    }
}
