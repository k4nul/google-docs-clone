use axum::http::StatusCode;
use axum_test::{TestServer, WsMessage};
use backend::{
    app::build_app,
    collab::{
        locator::{ResolvedRoom, RoomLocator, RoomLocatorError, RoomOwnerHint},
        rooms::RoomRegistry,
    },
    config::Config,
    state::AppState,
    storage::{DocumentSnapshot, FileSnapshotStore, InMemorySnapshotStore, SnapshotStore},
};
use serde_json::Value;
use std::{collections::HashMap, fs, sync::Arc, time::Duration};
use uuid::Uuid;
use yrs::{
    Doc, GetString, ReadTxn, StateVector, Text, Transact, Update,
    sync::{AwarenessUpdate, Message, SyncMessage, awareness::AwarenessUpdateEntry},
    updates::{decoder::Decode, encoder::Encode},
};

fn test_config() -> Config {
    Config {
        host: "127.0.0.1".to_owned(),
        port: 4000,
        frontend_origin: "http://localhost:3000".to_owned(),
        rust_log: "backend=debug".to_owned(),
        api_token: "test-admin-token".to_owned(),
        snapshot_store: "memory".to_owned(),
        snapshot_dir: "./data/test-snapshots".to_owned(),
        room_locator: "local".to_owned(),
        node_id: "test-node".to_owned(),
        room_owner_hints_path: None,
    }
}

fn temp_snapshot_dir(test_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("backend-{test_name}-{}", Uuid::new_v4()))
}

fn admin_auth_header(config: &Config) -> String {
    format!("Bearer {}", config.api_token)
}

fn document_auth_header(access_token: &str) -> String {
    format!("Bearer {access_token}")
}

#[derive(Debug, Default)]
struct RemoteRoomLocator;

impl RoomLocator for RemoteRoomLocator {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        Ok(ResolvedRoom::Remote(RoomOwnerHint {
            node_id: format!("node-for-{doc_id}"),
            base_url: Some("http://node-b.internal:4000".to_owned()),
        }))
    }
}

fn decode_sync_message(payload: impl AsRef<[u8]>) -> SyncMessage {
    let message = Message::decode_v1(payload.as_ref()).expect("websocket payload should decode");
    match message {
        Message::Sync(message) => message,
        other => panic!("expected sync message, received {other:?}"),
    }
}

#[tokio::test]
async fn health_endpoint_returns_ok_payload() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
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
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);
    let response = server
        .get("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    assert!(payload["documents"].as_array().is_some());
}

#[tokio::test]
async fn create_document_endpoint_creates_document_and_lists_it() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({
            "title": "Design notes"
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let payload = response.json::<Value>();
    let created_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned");
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned");
    assert_eq!(payload["document"]["title"].as_str(), Some("Design notes"));
    assert!(payload["document"]["created_at"].as_str().is_some());
    assert!(payload["document"]["updated_at"].as_str().is_some());

    let detail_response = server
        .get(&format!("/api/documents/{created_id}"))
        .add_header("Authorization", document_auth_header(access_token).as_str())
        .await;
    detail_response.assert_status_ok();

    let detail_payload = detail_response.json::<Value>();
    assert_eq!(detail_payload["document"]["id"].as_str(), Some(created_id));
    assert!(detail_payload["document"]["access_token"].is_null());

    let list_response = server
        .get("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .await;
    list_response.assert_status_ok();

    let list_payload = list_response.json::<Value>();
    let documents = list_payload["documents"]
        .as_array()
        .expect("documents should be returned as an array");

    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0]["id"].as_str(), Some(created_id));
    assert!(documents[0]["access_token"].is_null());
}

#[tokio::test]
async fn documents_endpoint_lists_snapshot_backed_documents_after_room_eviction() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Evicted but listed".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have an active room");
    assert_eq!(room.start_session(), 1);
    state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("idle room eviction should succeed");
    assert!(state.rooms().get(&document.id).is_none());

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    let documents = payload["documents"]
        .as_array()
        .expect("documents should be returned as an array");

    assert_eq!(documents.len(), 1);
    assert_eq!(
        documents[0]["id"].as_str(),
        Some(document.id.to_string().as_str())
    );
    assert_eq!(documents[0]["title"].as_str(), Some("Evicted but listed"));
}

#[tokio::test]
async fn delete_document_endpoint_removes_existing_document() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let create_response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({
            "title": "Disposable"
        }))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let created_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned")
        .to_owned();

    let delete_response = server
        .delete(&format!("/api/documents/{created_id}"))
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await;
    delete_response.assert_status(StatusCode::NO_CONTENT);

    let get_response = server
        .get(&format!("/api/documents/{created_id}"))
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await;
    get_response.assert_status_not_found();
}

#[tokio::test]
async fn delete_document_endpoint_rejects_documents_with_active_websocket_sessions() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Busy delete".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let websocket = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    let delete_response = server
        .delete(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    delete_response.assert_status(StatusCode::CONFLICT);
    let payload = delete_response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(
        payload["message"],
        format!(
            "document `{}` cannot be deleted while collaboration sessions are active",
            document.id
        )
    );

    websocket.close().await;
}

#[tokio::test]
async fn delete_document_endpoint_allows_delete_after_websocket_session_closes() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Delete after close".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let websocket = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    websocket.close().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let delete_response = server
        .delete(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;
    delete_response.assert_status(StatusCode::NO_CONTENT);

    let detail_response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;
    detail_response.assert_status_not_found();
}

#[tokio::test]
async fn document_detail_endpoint_rejects_missing_document_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);
    let doc_id = Uuid::nil();

    let response = server
        .get(&format!("/api/documents/{doc_id}"))
        .add_header("Authorization", "Bearer missing-doc-token")
        .await;

    response.assert_status_not_found();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "not_found");
    assert_eq!(
        payload["message"],
        format!("document `{doc_id}` was not found")
    );
}

#[tokio::test]
async fn document_detail_endpoint_rejects_non_local_room_owner() {
    let config = test_config();
    let state = AppState::with_snapshot_store_and_locator(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        Arc::new(InMemorySnapshotStore::new()),
        Arc::new(RemoteRoomLocator),
    )
    .expect("state should initialize with rejecting locator");
    let document = state
        .rooms()
        .create_document(Some("Remote owner".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status(StatusCode::CONFLICT);

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(
        payload["message"],
        format!(
            "document `{}` is owned by another collaboration node",
            document.id
        )
    );
    assert_eq!(
        payload["owner"]["node_id"],
        format!("node-for-{}", document.id)
    );
    assert_eq!(payload["owner"]["base_url"], "http://node-b.internal:4000");
}

#[tokio::test]
async fn websocket_endpoint_accepts_document_connections() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let create_response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({}))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let doc_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned")
        .to_owned();

    let websocket = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await
        .into_websocket()
        .await;

    websocket.close().await;
}

#[tokio::test]
async fn document_detail_endpoint_rejects_invalid_uuid_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
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
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
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
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let doc_id = Uuid::new_v4();

    let response = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", "http://evil.example")
        .add_header("Authorization", "Bearer test-doc-token")
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
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let response = server
        .get_websocket("/ws/not-a-uuid")
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header("Authorization", "Bearer test-doc-token")
        .await;

    response.assert_status_bad_request();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "bad_request");
    assert_eq!(
        payload["message"],
        "doc_id must be a valid UUID, received `not-a-uuid`"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_missing_document_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let doc_id = Uuid::new_v4();

    let response = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header("Authorization", "Bearer test-doc-token")
        .await;

    response.assert_status_not_found();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "not_found");
    assert_eq!(
        payload["message"],
        format!("document `{doc_id}` was not found")
    );
}

#[tokio::test]
async fn documents_endpoint_rejects_missing_admin_token() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let response = server.get("/api/documents").await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "unauthorized");
    assert_eq!(payload["message"], "Authorization header is required");
}

#[tokio::test]
async fn document_detail_endpoint_rejects_invalid_document_token() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let create_response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({
            "title": "Restricted"
        }))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let created_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned");

    let response = server
        .get(&format!("/api/documents/{created_id}"))
        .add_header("Authorization", "Bearer invalid-doc-token")
        .await;

    response.assert_status_forbidden();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "forbidden");
    assert_eq!(
        payload["message"],
        format!("provided token does not grant access to document `{created_id}`")
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_missing_document_token() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let create_response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({}))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let doc_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();

    let response = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", config.frontend_origin.as_str())
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "unauthorized");
    assert_eq!(payload["message"], "Authorization header is required");
}

#[tokio::test]
async fn websocket_endpoint_supports_yrs_sync_handshake_and_update_broadcast() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Provider compatibility".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have an active room");
    {
        let server_doc = room.awareness().write().await.doc().clone();
        let text = server_doc.get_or_insert_text("content");
        let mut txn = server_doc.transact_mut();
        text.insert(&mut txn, 0, "seed");
    }

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let mut first_client = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    first_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;

    let sync_reply = decode_sync_message(first_client.receive_bytes().await);
    let SyncMessage::SyncStep2(update) = sync_reply else {
        panic!("expected SyncStep2 during initial handshake");
    };

    let first_client_doc = Doc::new();
    let first_client_text = first_client_doc.get_or_insert_text("content");
    let mut first_client_txn = first_client_doc.transact_mut();
    first_client_txn
        .apply_update(Update::decode_v1(update.as_slice()).expect("sync payload should decode"));
    drop(first_client_txn);
    assert_eq!(
        first_client_text.get_string(&first_client_doc.transact()),
        "seed"
    );

    let mut second_client = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    second_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;
    let second_sync_reply = decode_sync_message(second_client.receive_bytes().await);
    let SyncMessage::SyncStep2(second_initial_update) = second_sync_reply else {
        panic!("expected SyncStep2 during second client handshake");
    };
    let second_client_doc = Doc::new();
    let second_client_text = second_client_doc.get_or_insert_text("content");
    let mut second_client_txn = second_client_doc.transact_mut();
    second_client_txn.apply_update(
        Update::decode_v1(second_initial_update.as_slice())
            .expect("second sync payload should decode"),
    );
    drop(second_client_txn);

    let mut update_txn = first_client_doc.transact_mut();
    first_client_text.insert(&mut update_txn, 4, " + provider");
    let client_update = update_txn.encode_update_v1();
    drop(update_txn);

    first_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::Update(client_update.clone()))
                .encode_v1()
                .into(),
        ))
        .await;

    let broadcast = decode_sync_message(second_client.receive_bytes().await);
    let SyncMessage::Update(update) = broadcast else {
        panic!("expected broadcast update for subscribed client");
    };
    let mut second_client_txn = second_client_doc.transact_mut();
    second_client_txn
        .apply_update(Update::decode_v1(update.as_slice()).expect("update payload should decode"));
    drop(second_client_txn);
    assert_eq!(
        second_client_text.get_string(&second_client_doc.transact()),
        "seed + provider"
    );

    first_client.close().await;
    second_client.close().await;
}

#[tokio::test]
async fn websocket_endpoint_rejects_invalid_awareness_payload_updates() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Awareness validation".to_owned()))
        .expect("document should be created");

    let app = build_app(&config, state.clone()).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let mut client = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    let invalid_awareness = AwarenessUpdate {
        clients: HashMap::from([(
            7,
            AwarenessUpdateEntry {
                clock: 1,
                json: r#"{"user":{"id":"user-7","name":"Kim","color":"blue"},"client":{"id":"session-3","kind":"editor"}}"#
                    .to_owned(),
            },
        )]),
    };

    client
        .send_message(WsMessage::Binary(
            Message::Awareness(invalid_awareness).encode_v1().into(),
        ))
        .await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let room = state
        .rooms()
        .get_or_restore(&document.id)
        .expect("room lookup should succeed")
        .expect("document room should restore after the invalid update path");
    let awareness_ref = room.awareness();
    let awareness = awareness_ref.read().await;

    assert!(!awareness.clients().contains_key(&7));

    client.close().await;
}

#[tokio::test]
async fn app_state_hydrates_snapshot_backed_rooms_on_startup() {
    let config = test_config();
    let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
    let bootstrap_registry = RoomRegistry::new(snapshot_store.clone());
    let document = bootstrap_registry
        .create_document(Some("Hydrated at startup".to_owned()))
        .expect("document should be created");
    let room = bootstrap_registry
        .get(&document.id)
        .expect("created document should have an active room");

    {
        let server_doc = room.awareness().write().await.doc().clone();
        let text = server_doc.get_or_insert_text("content");
        let mut txn = server_doc.transact_mut();
        text.insert(&mut txn, 0, "restored");
    }

    assert_eq!(room.start_session(), 1);
    let evicted = bootstrap_registry
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("idle room eviction should succeed");
    assert!(evicted);
    assert!(bootstrap_registry.get(&document.id).is_none());

    let state = AppState::with_snapshot_store(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        snapshot_store,
    )
    .expect("state should hydrate rooms from snapshot store");

    let hydrated_room = state
        .rooms()
        .get(&document.id)
        .expect("room should be present after startup hydration");
    let hydrated_doc = hydrated_room.awareness().read().await.doc().clone();
    let hydrated_text = hydrated_doc.get_or_insert_text("content");

    assert_eq!(
        hydrated_text.get_string(&hydrated_doc.transact()),
        "restored"
    );
}

#[tokio::test]
async fn app_state_uses_file_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("file-store");
    config.snapshot_store = "file".to_owned();
    config.snapshot_dir = snapshot_dir.to_string_lossy().into_owned();

    let state = AppState::from_config(&config).expect("state should initialize with file store");
    let document = state
        .rooms()
        .create_document(Some("Persisted to disk".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let evicted = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to disk on eviction");
    assert!(evicted);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted file snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_dir.join(format!("{}.json", document.id)).exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[tokio::test]
async fn app_state_with_file_store_skips_corrupt_snapshots_during_startup() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("file-store-corrupt-startup");
    config.snapshot_store = "file".to_owned();
    config.snapshot_dir = snapshot_dir.to_string_lossy().into_owned();

    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let valid_document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Healthy".to_owned()));
    let valid_update = Doc::new()
        .transact()
        .encode_state_as_update_v1(&StateVector::default());
    store
        .save_snapshot(DocumentSnapshot::new(valid_document.clone(), valid_update))
        .expect("valid snapshot should save");

    let corrupt_doc_id = Uuid::new_v4();
    fs::write(
        snapshot_dir.join(format!("{corrupt_doc_id}.json")),
        b"{not-json",
    )
    .expect("corrupt snapshot fixture should be written");

    let state = AppState::from_config(&config)
        .expect("startup hydration should continue past corrupt snapshots");
    let hydrated_documents = state
        .rooms()
        .list_documents()
        .expect("document catalog should still be available");

    assert!(state.rooms().get(&valid_document.id).is_some());
    assert_eq!(hydrated_documents, vec![valid_document]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[tokio::test]
async fn app_state_with_file_store_cleans_matching_stale_temp_snapshots_during_startup() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("file-store-stale-temp-startup");
    config.snapshot_store = "file".to_owned();
    config.snapshot_dir = snapshot_dir.to_string_lossy().into_owned();

    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let valid_document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Healthy".to_owned()));
    let valid_update = Doc::new()
        .transact()
        .encode_state_as_update_v1(&StateVector::default());
    store
        .save_snapshot(DocumentSnapshot::new(valid_document.clone(), valid_update))
        .expect("valid snapshot should save");

    let stale_temp_path =
        snapshot_dir.join(format!("{}.json.{}.tmp", valid_document.id, Uuid::new_v4()));
    fs::write(&stale_temp_path, br#"{"partial":true}"#)
        .expect("stale temp snapshot fixture should be written");

    let state = AppState::from_config(&config)
        .expect("startup hydration should clean stale temp files and restore valid snapshots");
    let hydrated_documents = state
        .rooms()
        .list_documents()
        .expect("document catalog should still be available");

    assert!(state.rooms().get(&valid_document.id).is_some());
    assert_eq!(hydrated_documents, vec![valid_document]);
    assert!(!stale_temp_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[tokio::test]
async fn app_state_with_file_store_cleans_orphan_stale_temp_snapshots_during_startup() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("file-store-orphan-stale-temp-startup");
    config.snapshot_store = "file".to_owned();
    config.snapshot_dir = snapshot_dir.to_string_lossy().into_owned();

    let orphan_doc_id = Uuid::new_v4();
    let stale_temp_path = snapshot_dir.join(format!("{orphan_doc_id}.json.{}.tmp", Uuid::new_v4()));
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    fs::write(&stale_temp_path, br#"{"partial":true}"#)
        .expect("stale temp snapshot fixture should be written");

    let state =
        AppState::from_config(&config).expect("startup hydration should clean orphan temp files");
    let hydrated_documents = state
        .rooms()
        .list_documents()
        .expect("document catalog should remain empty");

    assert!(state.rooms().get(&orphan_doc_id).is_none());
    assert!(hydrated_documents.is_empty());
    assert!(!stale_temp_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("file-store-unit");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Disk".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to file store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from file store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from file store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_replaces_existing_snapshot_without_leaking_temp_files() {
    let snapshot_dir = temp_snapshot_dir("file-store-atomic-save");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Atomic".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![1, 2, 3]))
        .expect("initial snapshot should save");
    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![9, 8, 7]))
        .expect("replacement snapshot should save");

    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from file store")
        .expect("snapshot should exist");
    let directory_entries = fs::read_dir(&snapshot_dir)
        .expect("snapshot directory should be readable")
        .map(|entry| entry.expect("snapshot entry should be readable").path())
        .collect::<Vec<_>>();
    let json_entries = directory_entries
        .iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .count();
    let temp_entries = directory_entries
        .iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("tmp"))
        .count();

    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![9, 8, 7]);
    assert_eq!(json_entries, 1);
    assert_eq!(temp_entries, 0);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_ignores_stale_temp_files_when_listing_documents() {
    let snapshot_dir = temp_snapshot_dir("file-store-stale-temp");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Atomic".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![4, 5, 6]))
        .expect("snapshot should save");

    let stale_temp_path = snapshot_dir.join(format!("{}.json.{}.tmp", document.id, Uuid::new_v4()));
    fs::write(&stale_temp_path, br#"{"partial":true}"#)
        .expect("stale temp snapshot fixture should be written");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should ignore stale temp files");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should still load from file store")
        .expect("snapshot should still exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![4, 5, 6]);
    assert!(stale_temp_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_delete_snapshot_removes_matching_stale_temp_files() {
    let snapshot_dir = temp_snapshot_dir("file-store-delete-stale-temp");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Atomic".to_owned()));
    let unrelated_doc_id = Uuid::new_v4();

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![4, 5, 6]))
        .expect("snapshot should save");

    let matching_temp_path =
        snapshot_dir.join(format!("{}.json.{}.tmp", document.id, Uuid::new_v4()));
    let unrelated_temp_path =
        snapshot_dir.join(format!("{}.json.{}.tmp", unrelated_doc_id, Uuid::new_v4()));
    fs::write(&matching_temp_path, br#"{"partial":true}"#)
        .expect("matching stale temp snapshot fixture should be written");
    fs::write(&unrelated_temp_path, br#"{"partial":true}"#)
        .expect("unrelated stale temp snapshot fixture should be written");

    store
        .delete_snapshot(&document.id)
        .expect("delete should remove snapshot and matching temp files");

    assert!(
        store
            .load_snapshot(&document.id)
            .expect("snapshot lookup should succeed")
            .is_none()
    );
    assert!(!matching_temp_path.exists());
    assert!(unrelated_temp_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[cfg(unix)]
#[test]
fn file_snapshot_store_preserves_previous_snapshot_when_atomic_replace_cannot_write_temp_file() {
    use std::os::unix::fs::PermissionsExt;

    let snapshot_dir = temp_snapshot_dir("file-store-atomic-save-failure");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Atomic".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![1, 2, 3]))
        .expect("initial snapshot should save");

    let original_permissions = fs::metadata(&snapshot_dir)
        .expect("snapshot directory metadata should be readable")
        .permissions();
    let mut readonly_permissions = original_permissions.clone();
    readonly_permissions.set_mode(0o555);
    fs::set_permissions(&snapshot_dir, readonly_permissions)
        .expect("snapshot directory should become read-only");

    let failed_save = store.save_snapshot(DocumentSnapshot::new(document.clone(), vec![9, 8, 7]));

    fs::set_permissions(&snapshot_dir, original_permissions)
        .expect("snapshot directory permissions should be restored");

    assert!(matches!(
        failed_save,
        Err(backend::storage::StorageError::Io(message)) if message.contains(".tmp")
    ));

    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("original snapshot should still load")
        .expect("original snapshot should still exist");
    let temp_entries = fs::read_dir(&snapshot_dir)
        .expect("snapshot directory should be readable")
        .map(|entry| entry.expect("snapshot entry should be readable").path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("tmp"))
        .count();

    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);
    assert_eq!(temp_entries, 0);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_skips_corrupt_snapshots_when_listing_documents() {
    let snapshot_dir = temp_snapshot_dir("file-store-corrupt-catalog");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Catalog".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![7, 8, 9]))
        .expect("valid snapshot should save");

    let corrupt_doc_id = Uuid::new_v4();
    fs::write(snapshot_dir.join(format!("{corrupt_doc_id}.json")), b"[]")
        .expect("corrupt snapshot fixture should be written");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should skip corrupt snapshots");
    let corrupt_snapshot_error = store
        .load_snapshot(&corrupt_doc_id)
        .expect_err("directly loading a corrupt snapshot should still fail");

    assert_eq!(listed_documents, vec![document]);
    assert!(matches!(
        corrupt_snapshot_error,
        backend::storage::StorageError::CorruptSnapshot(id) if id == corrupt_doc_id
    ));

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}
