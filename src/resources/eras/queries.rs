use futures_util::TryStreamExt;
use mongodb::{
    Collection, Database,
    bson::{Bson, Document, doc},
    options::FindOneOptions,
};

use crate::resources::eras::model::{
    EpisodeDto, EpisodeListItem, EpisodeSearchItem, EraDto, EraListItem, Reference,
};

pub enum EpisodeLookup {
    EraNotFound,
    EpisodeNotFound,
    Found(EpisodeDto),
}

fn eras_collection(db: &Database) -> Collection<Document> {
    db.collection("eras")
}

pub async fn list_eras(db: &Database, lang: &str) -> mongodb::error::Result<Vec<EraListItem>> {
    let mut projection = doc! {"_id": 1, "type": 1};
    projection.insert(format!("{lang}.name"), 1);
    projection.insert(format!("{lang}.label"), 1);
    projection.insert(format!("{lang}.episodes"), 1);

    let options =
        mongodb::options::FindOptions::builder().projection(projection).sort(doc! {"_id": 1}).build();
    let mut cursor = eras_collection(db).find(doc! {}, options).await?;

    let mut eras = Vec::new();
    while let Some(era) = cursor.try_next().await? {
        eras.push(parse_era_list_item(era, lang));
    }

    Ok(eras)
}

pub async fn find_era_by_id(
    db: &Database,
    era_id: &str,
    lang: &str,
) -> mongodb::error::Result<Option<EraDto>> {
    let mut projection = doc! {"_id": 1, "type": 1};
    projection.insert(lang, 1);
    let options = FindOneOptions::builder().projection(projection).build();
    let era = eras_collection(db).find_one(doc! {"_id": era_id}, options).await?;
    Ok(era.map(|doc| parse_era(doc, lang)))
}

pub async fn list_episodes_for_era(
    db: &Database,
    era_id: &str,
    lang: &str,
) -> mongodb::error::Result<Option<Vec<EpisodeListItem>>> {
    let mut projection = Document::new();
    projection.insert(format!("{lang}.episodes"), 1);
    let options = FindOneOptions::builder().projection(projection).build();
    let era = eras_collection(db).find_one(doc! {"_id": era_id}, options).await?;
    Ok(era.map(|doc| parse_episode_list(doc, lang)))
}

pub async fn find_episode_for_era(
    db: &Database,
    era_id: &str,
    episode_id: &str,
    lang: &str,
) -> mongodb::error::Result<EpisodeLookup> {
    let mut projection = Document::new();
    projection.insert(format!("{lang}.episodes"), 1);
    let options = FindOneOptions::builder().projection(projection).build();
    let era = eras_collection(db).find_one(doc! {"_id": era_id}, options).await?;
    let Some(era) = era else {
        return Ok(EpisodeLookup::EraNotFound);
    };

    for episode in episodes_for_lang(&era, lang) {
        if episode.id == episode_id {
            return Ok(EpisodeLookup::Found(episode));
        }
    }

    Ok(EpisodeLookup::EpisodeNotFound)
}

fn parse_era_list_item(doc: Document, lang: &str) -> EraListItem {
    let locale = get_document_for_lang(&doc, lang);
    let episodes = locale.and_then(|d| get_array(d, "episodes")).unwrap_or_default();

    EraListItem {
        id: get_string(&doc, "_id"),
        name: locale.and_then(|d| get_string_opt(d, "name")).unwrap_or_default(),
        label: locale.and_then(|d| get_string_opt(d, "label")).unwrap_or_default(),
        era_type: get_optional_string(&doc, "type"),
        episode_count: episodes.len(),
    }
}

pub async fn search_episodes_by_book(
    db: &Database,
    book: &str,
    lang: &str,
) -> mongodb::error::Result<Vec<EpisodeSearchItem>> {
    let episodes_path = format!("{lang}.episodes");
    let references_path = format!("{episodes_path}.references");
    let book_id_path = format!("{references_path}.book_id");
    let book_label_path = format!("{references_path}.book");
    let era_label_path = format!("${lang}.label");
    let episode_id_path = format!("${episodes_path}.id");
    let episode_label_path = format!("${episodes_path}.label");

    let mut match_book_id = Document::new();
    match_book_id.insert(book_id_path, book);
    let mut match_book_label = Document::new();
    match_book_label.insert(book_label_path, book);

    let pipeline = vec![
        doc! {"$unwind": format!("${episodes_path}")},
        doc! {"$unwind": format!("${references_path}")},
        doc! {"$match": {"$or": [match_book_id, match_book_label]}},
        doc! {
            "$group": {
                "_id": {
                    "era_id": "$_id",
                    "episode_id": episode_id_path
                },
                "era_label": {"$first": era_label_path},
                "episode_label": {"$first": episode_label_path}
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

fn get_string_opt(doc: &Document, key: &str) -> Option<String> {
    match doc.get(key) {
        Some(Bson::String(value)) => Some(value.clone()),
        Some(Bson::Null) | None => None,
        Some(other) => Some(other.to_string()),
    }
}

fn get_optional_string(doc: &Document, key: &str) -> Option<String> {
    get_string_opt(doc, key)
}

fn get_document_for_lang<'a>(doc: &'a Document, lang: &str) -> Option<&'a Document> {
    match doc.get(lang) {
        Some(Bson::Document(value)) => Some(value),
        _ => None,
    }
}

fn get_array(doc: &Document, key: &str) -> Option<Vec<Document>> {
    let Some(Bson::Array(items)) = doc.get(key) else {
        return None;
    };

    let mut docs = Vec::with_capacity(items.len());
    for item in items {
        if let Bson::Document(d) = item {
            docs.push(d.clone());
        }
    }
    Some(docs)
}

fn parse_era(doc: Document, lang: &str) -> EraDto {
    let locale = get_document_for_lang(&doc, lang);
    let episodes = episodes_for_lang(&doc, lang);
    let books = locale
        .and_then(|d| d.get("books"))
        .and_then(|b| b.as_array())
        .map(|arr| arr.iter().filter_map(Bson::as_str).map(str::to_owned).collect())
        .unwrap_or_default();

    EraDto {
        id: get_string(&doc, "_id"),
        name: locale.and_then(|d| get_string_opt(d, "name")).unwrap_or_default(),
        label: locale.and_then(|d| get_string_opt(d, "label")).unwrap_or_default(),
        era_type: get_optional_string(&doc, "type"),
        books,
        episodes,
    }
}

fn parse_episode_list(doc: Document, lang: &str) -> Vec<EpisodeListItem> {
    episodes_for_lang(&doc, lang)
        .into_iter()
        .map(|episode| EpisodeListItem {
            id: episode.id,
            name: episode.name,
            label: episode.label,
            reference_count: episode.references.len(),
        })
        .collect()
}

fn episodes_for_lang(doc: &Document, lang: &str) -> Vec<EpisodeDto> {
    let Some(locale) = get_document_for_lang(doc, lang) else {
        return Vec::new();
    };
    let Some(episodes) = get_array(locale, "episodes") else {
        return Vec::new();
    };

    episodes
        .into_iter()
        .map(|episode| EpisodeDto {
            id: get_string(&episode, "id"),
            name: get_string(&episode, "name"),
            label: get_string(&episode, "label"),
            references: parse_references(&episode),
        })
        .collect()
}

fn parse_references(episode: &Document) -> Vec<Reference> {
    let Some(Bson::Array(references)) = episode.get("references") else {
        return Vec::new();
    };

    references
        .iter()
        .filter_map(Bson::as_document)
        .map(|reference| Reference {
            book_id: get_string(reference, "book_id"),
            book: get_string(reference, "book"),
            chapters: reference
                .get("chapters")
                .and_then(Bson::as_array)
                .map(|items| {
                    items
                        .iter()
                        .map(|item| match item {
                            Bson::Int32(v) => *v,
                            Bson::Int64(v) => *v as i32,
                            Bson::Double(v) => *v as i32,
                            _ => 0,
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}
