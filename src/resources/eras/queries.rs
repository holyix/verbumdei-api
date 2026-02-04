use futures_util::TryStreamExt;
use mongodb::{
    Collection, Database,
    bson::{Bson, Document, doc},
    options::FindOneOptions,
};
use serde::Deserialize;

use crate::resources::eras::model::{
    EpisodeDto, EpisodeListItem, EpisodeSearchItem, Era, EraDto, EraListItem,
};

pub enum EpisodeLookup {
    EraNotFound,
    EpisodeNotFound,
    Found(EpisodeDto),
}

fn eras_collection(db: &Database) -> Collection<Era> {
    db.collection("eras")
}

#[derive(Deserialize)]
struct EraEpisodesOnly {
    #[serde(default)]
    episodes: Vec<crate::resources::eras::model::Episode>,
}

pub async fn list_eras(db: &Database) -> mongodb::error::Result<Vec<EraListItem>> {
    let pipeline = vec![
        doc! {
            "$project": {
                "_id": 1,
                "label": 1,
                "type": 1,
                "episode_count": { "$size": { "$ifNull": ["$episodes", []] } }
            }
        },
        doc! { "$sort": { "_id": 1 } },
    ];

    let mut cursor = eras_collection(db).aggregate(pipeline, None).await?;

    let mut eras = Vec::new();
    while let Some(era) = cursor.try_next().await? {
        eras.push(parse_era_list_item(era));
    }

    Ok(eras)
}

pub async fn find_era_by_id(db: &Database, era_id: &str) -> mongodb::error::Result<Option<EraDto>> {
    let era = eras_collection(db).find_one(doc! {"_id": era_id}, None).await?;
    Ok(era.map(EraDto::from))
}

pub async fn list_episodes_for_era(
    db: &Database,
    era_id: &str,
) -> mongodb::error::Result<Option<Vec<EpisodeListItem>>> {
    let options = FindOneOptions::builder().projection(doc! { "episodes": 1 }).build();
    let era: Option<EraEpisodesOnly> =
        db.collection("eras").find_one(doc! {"_id": era_id}, options).await?;
    Ok(era.map(|doc| doc.episodes.into_iter().map(EpisodeListItem::from).collect()))
}

pub async fn find_episode_for_era(
    db: &Database,
    era_id: &str,
    episode_id: &str,
) -> mongodb::error::Result<EpisodeLookup> {
    let options = FindOneOptions::builder().projection(doc! { "episodes": 1 }).build();
    let era: Option<EraEpisodesOnly> =
        db.collection("eras").find_one(doc! {"_id": era_id}, options).await?;
    let Some(era) = era else {
        return Ok(EpisodeLookup::EraNotFound);
    };

    for episode in era.episodes {
        if episode.id == episode_id {
            return Ok(EpisodeLookup::Found(EpisodeDto::from(episode)));
        }
    }

    Ok(EpisodeLookup::EpisodeNotFound)
}

fn parse_era_list_item(doc: Document) -> EraListItem {
    EraListItem {
        id: get_string(&doc, "_id"),
        label: get_string(&doc, "label"),
        era_type: get_optional_string(&doc, "type"),
        episode_count: get_u64(&doc, "episode_count") as usize,
    }
}

pub async fn search_episodes_by_book(
    db: &Database,
    book: &str,
) -> mongodb::error::Result<Vec<EpisodeSearchItem>> {
    let pipeline = vec![
        doc! {"$unwind": "$episodes"},
        doc! {"$unwind": "$episodes.references"},
        doc! {"$match": {"episodes.references.book": book}},
        doc! {
            "$group": {
                "_id": {
                    "era_id": "$_id",
                    "episode_id": "$episodes.id"
                },
                "era_label": {"$first": "$label"},
                "episode_label": {"$first": "$episodes.label"}
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "era_id": "$_id.era_id",
                "era_label": 1,
                "id": "$_id.episode_id",
                "label": "$episode_label"
            }
        },
        doc! {"$sort": {"era_id": 1, "id": 1}},
    ];

    let mut cursor = eras_collection(db).aggregate(pipeline, None).await?;
    let mut episodes = Vec::new();

    while let Some(item) = cursor.try_next().await? {
        episodes.push(parse_episode_search_item(item));
    }

    Ok(episodes)
}

fn parse_episode_search_item(doc: Document) -> EpisodeSearchItem {
    EpisodeSearchItem {
        era_id: get_string(&doc, "era_id"),
        era_label: get_string(&doc, "era_label"),
        id: get_string(&doc, "id"),
        label: get_string(&doc, "label"),
    }
}

fn get_string(doc: &Document, key: &str) -> String {
    match doc.get(key) {
        Some(Bson::String(value)) => value.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn get_optional_string(doc: &Document, key: &str) -> Option<String> {
    match doc.get(key) {
        Some(Bson::String(value)) => Some(value.clone()),
        Some(Bson::Null) | None => None,
        Some(other) => Some(other.to_string()),
    }
}

fn get_u64(doc: &Document, key: &str) -> u64 {
    match doc.get(key) {
        Some(Bson::Int32(v)) => *v as u64,
        Some(Bson::Int64(v)) => *v as u64,
        Some(Bson::Double(v)) => *v as u64,
        Some(other) => other.as_i64().unwrap_or_default() as u64,
        None => 0,
    }
}
