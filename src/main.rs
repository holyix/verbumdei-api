mod config;
mod routes;

use axum::{routing::get, Router};
use mongodb::{bson::doc, options::ClientOptions, Client, Database};
use tracing_subscriber::EnvFilter;

#[allow(dead_code)] // State includes DB handle for upcoming routes; currently unused.
#[derive(Clone)]
struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_tracing();

    let cfg = config::AppConfig::from_env();
    let db = init_mongo(&cfg).await.expect("failed to initialize MongoDB");

    tracing::info!("Starting server on {}", cfg.address());

    let app = Router::new()
        .route("/health", get(routes::health::health_check))
        .with_state(AppState { db });

    let listener = tokio::net::TcpListener::bind(cfg.address())
        .await
        .expect("failed to bind address");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("server error");
}

async fn init_mongo(cfg: &config::AppConfig) -> mongodb::error::Result<Database> {
    let mut client_options = ClientOptions::parse(&cfg.mongo_uri).await?;
    // Ensure app name for observability
    client_options.app_name = Some("verbumdei-api".to_string());

    let client = Client::with_options(client_options)?;
    let db = client.database(&cfg.mongo_db);

    // Basic connectivity check
    db.run_command(doc! { "ping": 1 }, None).await?;
    tracing::info!("Connected to MongoDB at {} (db: {})", cfg.mongo_uri, cfg.mongo_db);

    Ok(db)
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}
