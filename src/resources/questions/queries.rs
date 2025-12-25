use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, DateTime},
    options::FindOptions,
    Collection, Database,
};

use crate::resources::questions::model::{CreateQuestion, Question, QuestionDto};

pub async fn find_question_by_id(db: &Database, id: &str) -> mongodb::error::Result<Option<QuestionDto>> {
    let collection: Collection<Question> = db.collection("questions");
    let filter = match ObjectId::parse_str(id) {
        Ok(oid) => doc! { "_id": oid },
        Err(_) => doc! { "_id": id }, // allow string ids too
    };
    let res = collection.find_one(filter, None).await?;
    Ok(res.map(QuestionDto::from))
}

pub async fn insert_question(db: &Database, payload: CreateQuestion) -> mongodb::error::Result<QuestionDto> {
    let collection: Collection<Question> = db.collection("questions");

    let now = DateTime::now();
    let doc = Question {
        id: Bson::ObjectId(ObjectId::new()),
        stage: payload.stage,
        prompt: payload.prompt,
        options: payload.options,
        tags: payload.tags,
        created_at: now,
        updated_at: now,
    };

    collection.insert_one(&doc, None).await?;
    Ok(QuestionDto::from(doc))
}

pub async fn delete_question_by_id(db: &Database, id: &str) -> mongodb::error::Result<bool> {
    let collection: Collection<Question> = db.collection("questions");
    let filter = match ObjectId::parse_str(id) {
        Ok(oid) => doc! { "_id": oid },
        Err(_) => doc! { "_id": id },
    };

    let result = collection.delete_one(filter, None).await?;
    Ok(result.deleted_count > 0)
}

pub async fn list_questions(
    db: &Database,
    stage: Option<i32>,
    limit: i64,
    offset: u64,
) -> mongodb::error::Result<Vec<QuestionDto>> {
    let collection: Collection<Question> = db.collection("questions");

    let mut filter = doc! {};
    if let Some(stage) = stage {
        filter.insert("stage", stage);
    }

    let options = FindOptions::builder()
        .skip(Some(offset))
        .limit(Some(limit))
        .build();

    let mut cursor = collection.find(filter, options).await?;

    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await? {
        results.push(QuestionDto::from(doc));
    }
    Ok(results)
}
