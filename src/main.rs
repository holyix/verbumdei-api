mod config;
mod db;
mod resources;
mod routes;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_tracing();

    let cfg = config::AppConfig::from_env();
    let db = db::init_mongo(&cfg).await.expect("failed to initialize MongoDB");

    tracing::info!("Starting server on {}", cfg.address());

    let state = routes::api::ApiState {
        db,
    };
    let app = routes::api::router(state);

    let listener = tokio::net::TcpListener::bind(cfg.address()).await.expect("failed to bind address");
    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!("Server error: {err}");
    }
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).with_target(false).compact().init();
}
