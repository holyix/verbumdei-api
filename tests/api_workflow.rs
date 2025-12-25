use std::env;

use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

use verbumdei_api::{config::AppConfig, db, routes};

#[tokio::test]
async fn question_workflow_smoke() -> Result<(), Box<dyn std::error::Error>> {
    // Unique DB per test run to avoid collisions.
    let test_db = format!("verbumdei_test_{}", Uuid::new_v4());
    unsafe {
        env::set_var("MONGO_DB", &test_db);
        env::set_var("API_HOST", "127.0.0.1");
    }

    // Bind to an ephemeral port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    unsafe {
        env::set_var("API_PORT", addr.port().to_string());
    }

    let cfg = AppConfig::from_env();
    let db = db::init_mongo(&cfg).await?;
    let state = routes::api::ApiState {
        db,
    };
    let app = routes::api::router(state);

    // Start server in background.
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });

    let client = reqwest::Client::new();
    let base = format!("http://{}/v1", addr);

    // Create
    let valid_question = include_str!("fixtures/question_valid.json");

    let valid_payload: Value = serde_json::from_str(valid_question).expect("valid fixture");
    let create_res = client.post(format!("{}/questions", base)).json(&valid_payload).send().await?;

    assert_eq!(create_res.status(), StatusCode::CREATED);
    let created = create_res.json::<serde_json::Value>().await?;
    let id = created.get("id").and_then(|v| v.as_str()).ok_or("missing id")?.to_string();

    // Get
    let get_res = client.get(format!("{}/questions/{}", base, id)).send().await?;
    assert_eq!(get_res.status(), StatusCode::OK);

    // List
    let list_res = client.get(format!("{}/questions?limit=5&offset=0", base)).send().await?;
    assert_eq!(list_res.status(), StatusCode::OK);

    // Delete
    let delete_res = client.delete(format!("{}/questions/{}", base, id)).send().await?;
    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);

    // Cleanup server
    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn question_create_rejects_invalid_payload() -> Result<(), Box<dyn std::error::Error>> {
    // Unique DB per test run to avoid collisions.
    let test_db = format!("verbumdei_test_invalid_{}", Uuid::new_v4());
    unsafe {
        env::set_var("MONGO_DB", &test_db);
        env::set_var("API_HOST", "127.0.0.1");
    }

    // Bind to an ephemeral port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    unsafe {
        env::set_var("API_PORT", addr.port().to_string());
    }

    let cfg = AppConfig::from_env();
    let db = db::init_mongo(&cfg).await?;
    let state = routes::api::ApiState {
        db,
    };
    let app = routes::api::router(state);

    // Start server in background.
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });

    let client = reqwest::Client::new();
    let base = format!("http://{}/v1", addr);

    // Invalid payload: missing pt locale in prompt and only 3 options.
    let invalid_question = include_str!("fixtures/question_invalid.json");

    let invalid_payload: Value = serde_json::from_str(invalid_question).expect("invalid fixture");
    let create_res = client.post(format!("{}/questions", base)).json(&invalid_payload).send().await?;

    assert_eq!(create_res.status(), StatusCode::BAD_REQUEST);

    // Cleanup server
    server_handle.abort();
    Ok(())
}
