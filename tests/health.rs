use axum_test::TestServer;
use backend::{app::build_app, config::Config, state::AppState};
use serde_json::Value;
use uuid::Uuid;

fn test_config() -> Config {
    Config {
        host: "127.0.0.1".to_owned(),
        port: 4000,
        frontend_origin: "http://localhost:3000".to_owned(),
        rust_log: "backend=debug".to_owned(),
    }
}

#[tokio::test]
async fn health_endpoint_returns_ok_payload() {
    let config = test_config();
    let app = build_app(&config, AppState::from_config(&config)).expect("app should build");
    let server = TestServer::new(app);
    let response = server.get("/api/health").await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["service"], "backend");
    assert!(payload["timestamp"].as_str().is_some());
}

#[tokio::test]
async fn documents_endpoint_returns_documents_array() {
    let config = test_config();
    let app = build_app(&config, AppState::from_config(&config)).expect("app should build");
    let server = TestServer::new(app);
    let response = server.get("/api/documents").await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    assert!(payload["documents"].as_array().is_some());
}

#[tokio::test]
async fn document_detail_endpoint_returns_placeholder_for_requested_id() {
    let config = test_config();
    let app = build_app(&config, AppState::from_config(&config)).expect("app should build");
    let server = TestServer::new(app);
    let doc_id = Uuid::nil();
    let expected_id = doc_id.to_string();
    let expected_title = format!("Document {doc_id}");

    let response = server.get(&format!("/api/documents/{doc_id}")).await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    assert_eq!(
        payload["document"]["id"].as_str(),
        Some(expected_id.as_str())
    );
    assert_eq!(
        payload["document"]["title"].as_str(),
        Some(expected_title.as_str())
    );
    assert!(payload["document"]["created_at"].as_str().is_some());
    assert!(payload["document"]["updated_at"].as_str().is_some());

    let list_response = server.get("/api/documents").await;
    list_response.assert_status_ok();

    let list_payload = list_response.json::<Value>();
    let documents = list_payload["documents"]
        .as_array()
        .expect("documents should be returned as an array");

    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0]["id"].as_str(), Some(expected_id.as_str()));
}

#[tokio::test]
async fn websocket_endpoint_accepts_document_connections() {
    let config = test_config();
    let app = build_app(&config, AppState::from_config(&config)).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let doc_id = Uuid::new_v4();

    let websocket = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", config.frontend_origin.as_str())
        .await
        .into_websocket()
        .await;

    websocket.close().await;
}

#[tokio::test]
async fn document_detail_endpoint_rejects_invalid_uuid_with_json_error() {
    let config = test_config();
    let app = build_app(&config, AppState::from_config(&config)).expect("app should build");
    let server = TestServer::new(app);

    let response = server.get("/api/documents/not-a-uuid").await;

    response.assert_status_bad_request();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "bad_request");
    assert_eq!(
        payload["message"],
        "id must be a valid UUID, received `not-a-uuid`"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_missing_origin_with_json_error() {
    let config = test_config();
    let app = build_app(&config, AppState::from_config(&config)).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let doc_id = Uuid::new_v4();

    let response = server.get_websocket(&format!("/ws/{doc_id}")).await;

    response.assert_status_forbidden();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "forbidden");
    assert_eq!(
        payload["message"],
        "Origin header is required for websocket connections"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_disallowed_origin_with_json_error() {
    let config = test_config();
    let app = build_app(&config, AppState::from_config(&config)).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let doc_id = Uuid::new_v4();

    let response = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", "http://evil.example")
        .await;

    response.assert_status_forbidden();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "forbidden");
    assert_eq!(
        payload["message"],
        "Origin `http://evil.example` is not allowed for websocket connections"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_invalid_uuid_with_json_error() {
    let config = test_config();
    let app = build_app(&config, AppState::from_config(&config)).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let response = server
        .get_websocket("/ws/not-a-uuid")
        .add_header("Origin", config.frontend_origin.as_str())
        .await;

    response.assert_status_bad_request();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "bad_request");
    assert_eq!(
        payload["message"],
        "doc_id must be a valid UUID, received `not-a-uuid`"
    );
}
