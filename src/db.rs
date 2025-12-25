use mongodb::{Client, Database, bson::doc, options::ClientOptions};

use crate::config::AppConfig;

pub async fn init_mongo(cfg: &AppConfig) -> mongodb::error::Result<Database> {
    let mut client_options = ClientOptions::parse(&cfg.mongo_uri).await?;
    client_options.app_name = Some("verbumdei-api".to_string());

    let client = Client::with_options(client_options)?;
    let db = client.database(&cfg.mongo_db);

    db.run_command(doc! { "ping": 1 }, None).await?;
    tracing::info!("Connected to MongoDB at {} (db: {})", cfg.mongo_uri, cfg.mongo_db);

    Ok(db)
}
