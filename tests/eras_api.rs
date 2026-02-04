use std::env;

use mongodb::{
    Client,
    bson::{Document, doc},
};
use reqwest::StatusCode;
use uuid::Uuid;

use verbumdei_api::routes;

#[tokio::test]
async fn eras_endpoints_return_seeded_data() -> Result<(), Box<dyn std::error::Error>> {
    let test_app = TestApp::spawn().await?;

    let eras_res = test_app.client.get(format!("{}/v1/eras", test_app.base)).send().await?;
    assert_eq!(eras_res.status(), StatusCode::OK);
    let eras = eras_res.json::<Vec<serde_json::Value>>().await?;
    assert_eq!(eras.len(), 2);

    let era_res = test_app.client.get(format!("{}/v1/eras/creation", test_app.base)).send().await?;
    assert_eq!(era_res.status(), StatusCode::OK);
    let era = era_res.json::<serde_json::Value>().await?;
    assert_eq!(era.get("id").and_then(|v| v.as_str()), Some("creation"));

    let episodes_res =
        test_app.client.get(format!("{}/v1/eras/creation/episodes", test_app.base)).send().await?;
    assert_eq!(episodes_res.status(), StatusCode::OK);
    let episodes = episodes_res.json::<Vec<serde_json::Value>>().await?;
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].get("id").and_then(|v| v.as_str()), Some("world"));

    Ok(())
}

#[tokio::test]
async fn episodes_search_requires_book_and_filters_by_book() -> Result<(), Box<dyn std::error::Error>> {
    let test_app = TestApp::spawn().await?;

    let missing_book_res = test_app.client.get(format!("{}/v1/episodes", test_app.base)).send().await?;
    assert_eq!(missing_book_res.status(), StatusCode::BAD_REQUEST);

    let search_res =
        test_app.client.get(format!("{}/v1/episodes?book=Genesis", test_app.base)).send().await?;
    assert_eq!(search_res.status(), StatusCode::OK);
    let episodes = search_res.json::<Vec<serde_json::Value>>().await?;
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].get("id").and_then(|v| v.as_str()), Some("world"));

    Ok(())
}

struct TestApp {
    base: String,
    client: reqwest::Client,
    _db_guard: DbGuard,
    server_handle: tokio::task::JoinHandle<()>,
}

impl TestApp {
    async fn spawn() -> Result<Self, Box<dyn std::error::Error>> {
        let mongo_uri =
            env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://127.0.0.1:27017".to_string());
        let db_name = format!("verbumdei_test_eras_{}", Uuid::new_v4());
        let _db_guard = DbGuard::new(mongo_uri.clone(), db_name.clone());

        let mongo = Client::with_uri_str(&mongo_uri).await?;
        let db = mongo.database(&db_name);
        seed_eras(&db).await?;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        let state = routes::api::ApiState {
            db,
        };
        let app = routes::api::router(state);

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server error");
        });

        Ok(Self {
            base: format!("http://{}", addr),
            client: reqwest::Client::new(),
            _db_guard,
            server_handle,
        })
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}

struct DbGuard {
    uri: String,
    name: String,
}

impl DbGuard {
    fn new(uri: String, name: String) -> Self {
        Self {
            uri,
            name,
        }
    }
}

impl Drop for DbGuard {
    fn drop(&mut self) {
        let uri = self.uri.clone();
        let name = self.name.clone();
        let fut = async move {
            if let Ok(client) = Client::with_uri_str(&uri).await {
                let _ = client.database(&name).drop(None).await;
            }
        };
        tokio::runtime::Handle::current().spawn(fut);
    }
}

async fn seed_eras(db: &mongodb::Database) -> mongodb::error::Result<()> {
    let docs: Vec<Document> = vec![
        doc! {
            "_id": "creation",
            "label": "Creation",
            "books": ["Genesis"],
            "episodes": [
                {
                    "id": "world",
                    "label": "Creation of the World",
                    "references": [{"book": "Genesis", "chapters": [1]}]
                }
            ]
        },
        doc! {
            "_id": "exodus",
            "label": "Exodus",
            "books": ["Exodus"],
            "episodes": [
                {
                    "id": "moses",
                    "label": "Moses",
                    "references": [{"book": "Exodus", "chapters": [3]}]
                }
            ]
        },
    ];

    db.collection::<Document>("eras").insert_many(docs, None).await?;
    Ok(())
}
