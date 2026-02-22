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
    assert_eq!(eras[0].get("name").and_then(|v| v.as_str()), Some("Creation"));
    assert_eq!(eras[0].get("order").and_then(|v| v.as_i64()), Some(10));

    let era_res = test_app.client.get(format!("{}/v1/eras/creation", test_app.base)).send().await?;
    assert_eq!(era_res.status(), StatusCode::OK);
    let era = era_res.json::<serde_json::Value>().await?;
    assert_eq!(era.get("id").and_then(|v| v.as_str()), Some("creation"));
    assert_eq!(era.get("name").and_then(|v| v.as_str()), Some("Creation"));

    let episodes_res =
        test_app.client.get(format!("{}/v1/eras/creation/episodes", test_app.base)).send().await?;
    assert_eq!(episodes_res.status(), StatusCode::OK);
    let episodes = episodes_res.json::<Vec<serde_json::Value>>().await?;
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].get("id").and_then(|v| v.as_str()), Some("world"));
    assert_eq!(episodes[0].get("name").and_then(|v| v.as_str()), Some("World"));
    assert_eq!(episodes[0].get("order").and_then(|v| v.as_i64()), Some(10));

    Ok(())
}

#[tokio::test]
async fn eras_endpoints_resolve_lang_from_query_and_header() -> Result<(), Box<dyn std::error::Error>> {
    let test_app = TestApp::spawn().await?;

    let es_res = test_app.client.get(format!("{}/v1/eras?lang=es", test_app.base)).send().await?;
    assert_eq!(es_res.status(), StatusCode::OK);
    let eras = es_res.json::<Vec<serde_json::Value>>().await?;
    assert_eq!(eras[0].get("name").and_then(|v| v.as_str()), Some("Creación"));

    let sv_res = test_app
        .client
        .get(format!("{}/v1/eras/exodus/episodes/sinai", test_app.base))
        .header("Accept-Language", "fr-FR, sv-SE;q=0.9, en;q=0.8")
        .send()
        .await?;
    assert_eq!(sv_res.status(), StatusCode::OK);
    let episode = sv_res.json::<serde_json::Value>().await?;
    assert_eq!(episode.get("name").and_then(|v| v.as_str()), Some("Sinai"));
    assert_eq!(
        episode
            .get("references")
            .and_then(|refs| refs.as_array())
            .and_then(|refs| refs.first())
            .and_then(|r| r.get("book"))
            .and_then(|v| v.as_str()),
        Some("Andra Mosebok")
    );

    let fallback_res =
        test_app.client.get(format!("{}/v1/eras/exodus?lang=de", test_app.base)).send().await?;
    assert_eq!(fallback_res.status(), StatusCode::OK);
    let era = fallback_res.json::<serde_json::Value>().await?;
    assert_eq!(era.get("name").and_then(|v| v.as_str()), Some("Exodus"));

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
            "en": {
                "id": "creation",
                "name": "Creation",
                "label": "Creation",
                "order": 10,
                "books": ["Genesis"],
                "episodes": [
                    {
                        "id": "world",
                        "name": "World",
                        "label": "Creation of the World",
                        "order": 10,
                        "references": [{"book_id": "Genesis", "book": "Genesis", "chapters": [1]}]
                    }
                ]
            },
            "es": {
                "id": "creation",
                "name": "Creación",
                "label": "Creación",
                "order": 10,
                "books": ["Génesis"],
                "episodes": [
                    {
                        "id": "world",
                        "name": "Mundo",
                        "label": "Creación del Mundo",
                        "order": 10,
                        "references": [{"book_id": "Genesis", "book": "Génesis", "chapters": [1]}]
                    }
                ]
            },
            "pt": {
                "id": "creation",
                "name": "Criação",
                "label": "Criação",
                "order": 10,
                "books": ["Gênesis"],
                "episodes": [
                    {
                        "id": "world",
                        "name": "Mundo",
                        "label": "Criação do Mundo",
                        "order": 10,
                        "references": [{"book_id": "Genesis", "book": "Gênesis", "chapters": [1]}]
                    }
                ]
            },
            "sv": {
                "id": "creation",
                "name": "Skapelsen",
                "label": "Skapelsen",
                "order": 10,
                "books": ["Första Mosebok"],
                "episodes": [
                    {
                        "id": "world",
                        "name": "Världen",
                        "label": "Världens skapelse",
                        "order": 10,
                        "references": [{"book_id": "Genesis", "book": "Första Mosebok", "chapters": [1]}]
                    }
                ]
            }
        },
        doc! {
            "_id": "exodus",
            "en": {
                "id": "exodus",
                "name": "Exodus",
                "label": "Exodus and Sinai Covenant",
                "order": 20,
                "books": ["Exodus"],
                "episodes": [
                    {
                        "id": "moses",
                        "name": "Moses",
                        "label": "Moses and His Calling",
                        "order": 10,
                        "references": [{"book_id": "Exodus", "book": "Exodus", "chapters": [3]}]
                    },
                    {
                        "id": "sinai",
                        "name": "Sinai",
                        "label": "The Sinai Covenant",
                        "order": 20,
                        "references": [{"book_id": "Exodus", "book": "Exodus", "chapters": [19,20]}]
                    }
                ]
            },
            "es": {
                "id": "exodus",
                "name": "Éxodo",
                "label": "Éxodo y la Alianza del Sinaí",
                "order": 20,
                "books": ["Éxodo"],
                "episodes": [
                    {
                        "id": "moses",
                        "name": "Moisés",
                        "label": "Moisés y su Llamado",
                        "order": 10,
                        "references": [{"book_id": "Exodus", "book": "Éxodo", "chapters": [3]}]
                    },
                    {
                        "id": "sinai",
                        "name": "Sinaí",
                        "label": "La Alianza en el Sinaí",
                        "order": 20,
                        "references": [{"book_id": "Exodus", "book": "Éxodo", "chapters": [19,20]}]
                    }
                ]
            },
            "pt": {
                "id": "exodus",
                "name": "Êxodo",
                "label": "Êxodo e a Aliança do Sinai",
                "order": 20,
                "books": ["Êxodo"],
                "episodes": [
                    {
                        "id": "moses",
                        "name": "Moisés",
                        "label": "Moisés e seu Chamado",
                        "order": 10,
                        "references": [{"book_id": "Exodus", "book": "Êxodo", "chapters": [3]}]
                    },
                    {
                        "id": "sinai",
                        "name": "Sinai",
                        "label": "A Aliança no Sinai",
                        "order": 20,
                        "references": [{"book_id": "Exodus", "book": "Êxodo", "chapters": [19,20]}]
                    }
                ]
            },
            "sv": {
                "id": "exodus",
                "name": "Exodus",
                "label": "Uttåget och Sinai-förbundet",
                "order": 20,
                "books": ["Andra Mosebok"],
                "episodes": [
                    {
                        "id": "moses",
                        "name": "Mose",
                        "label": "Mose och hans kallelse",
                        "order": 10,
                        "references": [{"book_id": "Exodus", "book": "Andra Mosebok", "chapters": [3]}]
                    },
                    {
                        "id": "sinai",
                        "name": "Sinai",
                        "label": "Förbundet vid Sinai",
                        "order": 20,
                        "references": [{"book_id": "Exodus", "book": "Andra Mosebok", "chapters": [19,20]}]
                    }
                ]
            }
        },
    ];

    db.collection::<Document>("eras").insert_many(docs, None).await?;
    Ok(())
}
