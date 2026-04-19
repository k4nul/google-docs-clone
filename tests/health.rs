use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_test::{TestServer, WsMessage};
use backend::{
    app::build_app,
    collab::{
        coordinator::{RoomCoordinator, RoomCoordinatorError},
        locator::{ResolvedRoom, RoomLocator, RoomLocatorError, RoomOwnerHint},
        rooms::RoomRegistry,
    },
    config::Config,
    errors::AppError,
    state::AppState,
    storage::{
        DocumentSnapshot, FileSnapshotStore, FjallSnapshotStore, HeedSnapshotStore,
        InMemorySnapshotStore, JammdbSnapshotStore, ManagedSnapshotStore, PersySnapshotStore,
        RedbSnapshotStore, S3SnapshotStore, SledSnapshotStore, SnapshotStore, SqliteSnapshotStore,
    },
};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{net::TcpListener, task::JoinHandle};
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
        snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
        snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
        snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
        snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
        snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
        snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
        snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
        snapshot_s3_endpoint: None,
        snapshot_s3_region: "us-east-1".to_owned(),
        snapshot_s3_bucket: None,
        snapshot_s3_prefix: "snapshots/".to_owned(),
        snapshot_s3_access_key_id: None,
        snapshot_s3_secret_access_key: None,
        snapshot_s3_session_token: None,
        snapshot_s3_timeout_secs: 5,
        snapshot_s3_path_style: true,
        snapshot_managed_base_url: None,
        snapshot_managed_auth_token: None,
        snapshot_managed_timeout_secs: 5,
        room_locator: "local".to_owned(),
        room_coordinator: "noop".to_owned(),
        room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
        room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
        room_coordinator_heartbeat_interval_secs: 10,
        room_coordinator_lease_ttl_secs: 30,
        room_coordination_managed_base_url: None,
        room_coordination_managed_auth_token: None,
        room_coordination_managed_timeout_secs: 5,
        node_id: "test-node".to_owned(),
        node_base_url: None,
        room_owner_hints_path: None,
    }
}

fn temp_snapshot_dir(test_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("backend-{test_name}-{}", Uuid::new_v4()))
}

fn configure_shared_sqlite_collaboration(
    config: &mut Config,
    root: &std::path::Path,
    node_id: &str,
    node_base_url: &str,
) {
    config.snapshot_store = "sqlite".to_owned();
    config.snapshot_sqlite_path = root
        .join("snapshots.sqlite3")
        .to_string_lossy()
        .into_owned();
    config.room_locator = "sqlite".to_owned();
    config.room_coordinator = "sqlite".to_owned();
    config.room_coordinator_sqlite_path = root
        .join("room-coordinator.sqlite3")
        .to_string_lossy()
        .into_owned();
    config.node_id = node_id.to_owned();
    config.node_base_url = Some(node_base_url.to_owned());
}

fn configure_managed_snapshot_store(
    config: &mut Config,
    managed_base_url: &str,
    managed_auth_token: Option<&str>,
) {
    config.snapshot_store = "managed".to_owned();
    config.snapshot_managed_base_url = Some(managed_base_url.to_owned());
    config.snapshot_managed_auth_token = managed_auth_token.map(str::to_owned);
    config.snapshot_managed_timeout_secs = 5;
}

fn configure_s3_snapshot_store(
    config: &mut Config,
    endpoint: &str,
    bucket: &str,
    access_key_id: &str,
    secret_access_key: &str,
) {
    config.snapshot_store = "s3".to_owned();
    config.snapshot_s3_endpoint = Some(endpoint.to_owned());
    config.snapshot_s3_region = "us-east-1".to_owned();
    config.snapshot_s3_bucket = Some(bucket.to_owned());
    config.snapshot_s3_prefix = "snapshots/test-suite/".to_owned();
    config.snapshot_s3_access_key_id = Some(access_key_id.to_owned());
    config.snapshot_s3_secret_access_key = Some(secret_access_key.to_owned());
    config.snapshot_s3_session_token = None;
    config.snapshot_s3_timeout_secs = 5;
    config.snapshot_s3_path_style = true;
}

fn configure_redb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "redb".to_owned();
    config.snapshot_redb_path = root.join("snapshots.redb").to_string_lossy().into_owned();
}

fn configure_fjall_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "fjall".to_owned();
    config.snapshot_fjall_path = root.join("snapshots.fjall").to_string_lossy().into_owned();
}

fn configure_persy_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "persy".to_owned();
    config.snapshot_persy_path = root.join("snapshots.persy").to_string_lossy().into_owned();
}

fn configure_jammdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "jammdb".to_owned();
    config.snapshot_jammdb_path = root.join("snapshots.jammdb").to_string_lossy().into_owned();
}

fn configure_heed_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "heed".to_owned();
    config.snapshot_heed_path = root.join("snapshots.heed").to_string_lossy().into_owned();
}

fn configure_sled_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "sled".to_owned();
    config.snapshot_sled_path = root.join("snapshots.sled").to_string_lossy().into_owned();
}

fn configure_managed_coordination_with_shared_sqlite_snapshots(
    config: &mut Config,
    root: &std::path::Path,
    node_id: &str,
    node_base_url: &str,
    managed_base_url: &str,
    managed_auth_token: Option<&str>,
) {
    config.snapshot_store = "sqlite".to_owned();
    config.snapshot_sqlite_path = root
        .join("snapshots.sqlite3")
        .to_string_lossy()
        .into_owned();
    config.room_locator = "managed".to_owned();
    config.room_coordinator = "managed".to_owned();
    config.room_coordination_managed_base_url = Some(managed_base_url.to_owned());
    config.room_coordination_managed_auth_token = managed_auth_token.map(str::to_owned);
    config.room_coordinator_heartbeat_interval_secs = 1;
    config.room_coordinator_lease_ttl_secs = 3;
    config.node_id = node_id.to_owned();
    config.node_base_url = Some(node_base_url.to_owned());
}

fn configure_managed_coordination_with_managed_snapshots(
    config: &mut Config,
    node_id: &str,
    node_base_url: &str,
    coordination_base_url: &str,
    snapshot_base_url: &str,
    managed_auth_token: Option<&str>,
) {
    configure_managed_snapshot_store(config, snapshot_base_url, managed_auth_token);
    config.room_locator = "managed".to_owned();
    config.room_coordinator = "managed".to_owned();
    config.room_coordination_managed_base_url = Some(coordination_base_url.to_owned());
    config.room_coordination_managed_auth_token = managed_auth_token.map(str::to_owned);
    config.room_coordinator_heartbeat_interval_secs = 1;
    config.room_coordinator_lease_ttl_secs = 3;
    config.node_id = node_id.to_owned();
    config.node_base_url = Some(node_base_url.to_owned());
}

fn admin_auth_header(config: &Config) -> String {
    format!("Bearer {}", config.api_token)
}

fn document_auth_header(access_token: &str) -> String {
    format!("Bearer {access_token}")
}

#[derive(Debug, Clone, Default)]
struct MockS3ServiceState {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    last_authorization: Arc<Mutex<Option<String>>>,
}

impl MockS3ServiceState {
    fn object(&self, key: &str) -> Option<Vec<u8>> {
        self.objects
            .lock()
            .expect("mock s3 object store should not be poisoned")
            .get(key)
            .cloned()
    }

    fn last_authorization(&self) -> Option<String> {
        self.last_authorization
            .lock()
            .expect("mock s3 auth state should not be poisoned")
            .clone()
    }
}

struct MockS3Harness {
    state: MockS3ServiceState,
    endpoint: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
    task: JoinHandle<()>,
}

impl Drop for MockS3Harness {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn spawn_mock_s3_snapshot_service() -> MockS3Harness {
    let state = MockS3ServiceState::default();
    let bucket = "backend-test-snapshots".to_owned();
    let access_key_id = "test-access-key".to_owned();
    let secret_access_key = "test-secret-key".to_owned();
    let app = Router::new()
        .fallback(mock_s3_dispatch)
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("mock s3 listener should bind");
    let addr = listener
        .local_addr()
        .expect("mock s3 listener should expose local addr");
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("mock s3 service should serve");
    });

    MockS3Harness {
        state,
        endpoint: format!("http://{addr}"),
        bucket,
        access_key_id,
        secret_access_key,
        task,
    }
}

async fn mock_s3_dispatch(
    State(state): State<MockS3ServiceState>,
    method: Method,
    uri: Uri,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let authorization = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    *state
        .last_authorization
        .lock()
        .expect("mock s3 auth state should not be poisoned") = authorization;

    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        return mock_s3_xml_error(StatusCode::NOT_FOUND, "NoSuchBucket", "bucket not found");
    }

    let (bucket, key) = match path.split_once('/') {
        Some((bucket, "")) => (bucket, None),
        Some((bucket, key)) => (bucket, Some(key)),
        None => (path, None),
    };

    if bucket.trim().is_empty() {
        return mock_s3_xml_error(StatusCode::NOT_FOUND, "NoSuchBucket", "bucket not found");
    }

    if key.is_none() && query.get("list-type").map(String::as_str) == Some("2") {
        let prefix = query.get("prefix").cloned().unwrap_or_default();
        let mut objects = state
            .objects
            .lock()
            .expect("mock s3 object store should not be poisoned")
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(key, bytes)| (key.clone(), bytes.len()))
            .collect::<Vec<_>>();
        objects.sort_by(|left, right| left.0.cmp(&right.0));

        let contents = objects
            .into_iter()
            .map(|(key, size)| {
                format!(
                    "<Contents><Key>{key}</Key><Size>{size}</Size><StorageClass>STANDARD</StorageClass></Contents>"
                )
            })
            .collect::<String>();
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListBucketResult><Name>{bucket}</Name><Prefix>{prefix}</Prefix><KeyCount>{key_count}</KeyCount><MaxKeys>1000</MaxKeys><IsTruncated>false</IsTruncated>{contents}</ListBucketResult>",
            key_count = contents.matches("<Contents>").count(),
        );
        return mock_s3_xml_response(StatusCode::OK, xml);
    }

    let Some(key) = key else {
        return mock_s3_xml_error(StatusCode::NOT_FOUND, "NoSuchKey", "object not found");
    };

    match method {
        Method::GET => match state.object(key) {
            Some(bytes) => (StatusCode::OK, bytes).into_response(),
            None => mock_s3_xml_error(StatusCode::NOT_FOUND, "NoSuchKey", "object not found"),
        },
        Method::PUT => {
            state
                .objects
                .lock()
                .expect("mock s3 object store should not be poisoned")
                .insert(key.to_owned(), body.to_vec());
            StatusCode::OK.into_response()
        }
        Method::DELETE => {
            state
                .objects
                .lock()
                .expect("mock s3 object store should not be poisoned")
                .remove(key);
            StatusCode::NO_CONTENT.into_response()
        }
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    }
}

fn mock_s3_xml_response(status: StatusCode, body: String) -> Response {
    let mut response = (status, body).into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/xml"),
    );
    response
}

fn mock_s3_xml_error(status: StatusCode, code: &str, message: &str) -> Response {
    mock_s3_xml_response(
        status,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Error><Code>{code}</Code><Message>{message}</Message></Error>"
        ),
    )
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

#[derive(Debug, Default)]
struct RecordingRoomCoordinator {
    events: Mutex<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockManagedLeaseRecord {
    doc_id: Uuid,
    node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lease_id: Option<Uuid>,
    #[serde(default)]
    epoch: u64,
    activated_at: chrono::DateTime<Utc>,
    #[serde(default, alias = "updated_at", skip_serializing_if = "Option::is_none")]
    renewed_at: Option<chrono::DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct MockManagedAcquireRequest {
    node_id: String,
    base_url: Option<String>,
    lease_ttl_secs: u64,
}

#[derive(Debug, Deserialize)]
struct MockManagedRenewRequest {
    node_id: String,
    lease_id: Uuid,
    epoch: u64,
    lease_ttl_secs: u64,
}

#[derive(Debug, Deserialize)]
struct MockManagedReleaseRequest {
    node_id: String,
    lease_id: Uuid,
    epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockManagedSnapshotPayload {
    document: MockManagedSnapshotDocument,
    update: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockManagedSnapshotDocument {
    id: Uuid,
    title: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    access_token: String,
}

#[derive(Debug, Serialize)]
struct MockManagedSnapshotCatalogResponse {
    documents: Vec<MockManagedSnapshotDocument>,
}

impl From<DocumentSnapshot> for MockManagedSnapshotPayload {
    fn from(snapshot: DocumentSnapshot) -> Self {
        let document = snapshot.document;
        let access_token = document.access_token().to_owned();
        Self {
            document: MockManagedSnapshotDocument {
                id: document.id,
                title: document.title,
                created_at: document.created_at,
                updated_at: document.updated_at,
                access_token,
            },
            update: snapshot.update,
        }
    }
}

#[derive(Debug, Clone)]
struct MockManagedCoordinationServiceState {
    leases: Arc<Mutex<HashMap<Uuid, MockManagedLeaseRecord>>>,
    snapshots: Arc<Mutex<HashMap<Uuid, MockManagedSnapshotPayload>>>,
    auth_token: Option<String>,
}

impl MockManagedCoordinationServiceState {
    fn lease(&self, doc_id: &Uuid) -> Option<MockManagedLeaseRecord> {
        self.leases
            .lock()
            .expect("managed coordination lease store should not be poisoned")
            .get(doc_id)
            .cloned()
    }

    fn snapshot(&self, doc_id: &Uuid) -> Option<MockManagedSnapshotPayload> {
        self.snapshots
            .lock()
            .expect("managed snapshot store should not be poisoned")
            .get(doc_id)
            .cloned()
    }
}

struct MockManagedCoordinationHarness {
    state: MockManagedCoordinationServiceState,
    base_url: String,
    snapshot_base_url: String,
    task: JoinHandle<()>,
}

impl Drop for MockManagedCoordinationHarness {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn spawn_mock_managed_coordination_service(
    auth_token: Option<&str>,
) -> MockManagedCoordinationHarness {
    let state = MockManagedCoordinationServiceState {
        leases: Arc::new(Mutex::new(HashMap::new())),
        snapshots: Arc::new(Mutex::new(HashMap::new())),
        auth_token: auth_token.map(str::to_owned),
    };
    let app = Router::new()
        .route("/coord/v1/leases/{doc_id}", get(mock_managed_lookup_lease))
        .route(
            "/coord/v1/leases/{doc_id}/acquire",
            post(mock_managed_acquire_lease),
        )
        .route(
            "/coord/v1/leases/{doc_id}/renew",
            post(mock_managed_renew_lease),
        )
        .route(
            "/coord/v1/leases/{doc_id}/release",
            post(mock_managed_release_lease),
        )
        .route("/snapshot/v1/snapshots", get(mock_managed_list_snapshots))
        .route(
            "/snapshot/v1/snapshots/{doc_id}",
            get(mock_managed_get_snapshot)
                .put(mock_managed_put_snapshot)
                .delete(mock_managed_delete_snapshot),
        )
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("managed coordination listener should bind");
    let addr = listener
        .local_addr()
        .expect("managed coordination listener should expose local addr");
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("managed coordination service should serve");
    });

    MockManagedCoordinationHarness {
        state,
        base_url: format!("http://{addr}/coord"),
        snapshot_base_url: format!("http://{addr}/snapshot"),
        task,
    }
}

fn mock_managed_authorize(
    headers: &HeaderMap,
    state: &MockManagedCoordinationServiceState,
) -> Result<(), StatusCode> {
    let Some(expected_auth_token) = state.auth_token.as_deref() else {
        return Ok(());
    };
    let Some(header_value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Ok(header_value) = header_value.to_str() else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if header_value == format!("Bearer {expected_auth_token}") {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn mock_managed_lookup_lease(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    match state.lease(&doc_id) {
        Some(lease) => (StatusCode::OK, Json(lease)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn mock_managed_acquire_lease(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<MockManagedAcquireRequest>,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    let now = Utc::now();
    let ttl = ChronoDuration::seconds(payload.lease_ttl_secs as i64);
    let mut leases = state
        .leases
        .lock()
        .expect("managed coordination lease store should not be poisoned");

    if let Some(existing) = leases.get(&doc_id) {
        let active_remote_owner = existing.node_id.trim() != payload.node_id.trim()
            && existing
                .expires_at
                .map(|expires_at| expires_at > now)
                .unwrap_or(true);
        if active_remote_owner {
            return (StatusCode::CONFLICT, Json(existing.clone())).into_response();
        }
    }

    let epoch = leases
        .get(&doc_id)
        .map(|lease| lease.epoch.saturating_add(1))
        .unwrap_or(1);
    let lease = MockManagedLeaseRecord {
        doc_id,
        node_id: payload.node_id.trim().to_owned(),
        base_url: payload.base_url,
        lease_id: Some(Uuid::new_v4()),
        epoch,
        activated_at: now,
        renewed_at: Some(now),
        expires_at: Some(now + ttl),
    };
    leases.insert(doc_id, lease.clone());

    (StatusCode::OK, Json(lease)).into_response()
}

async fn mock_managed_renew_lease(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<MockManagedRenewRequest>,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    let now = Utc::now();
    let ttl = ChronoDuration::seconds(payload.lease_ttl_secs as i64);
    let mut leases = state
        .leases
        .lock()
        .expect("managed coordination lease store should not be poisoned");
    let Some(existing) = leases.get_mut(&doc_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if existing.node_id.trim() != payload.node_id.trim()
        || existing.lease_id != Some(payload.lease_id)
        || existing.epoch != payload.epoch
    {
        return (StatusCode::CONFLICT, Json(existing.clone())).into_response();
    }

    existing.renewed_at = Some(now);
    existing.expires_at = Some(now + ttl);
    (StatusCode::OK, Json(existing.clone())).into_response()
}

async fn mock_managed_release_lease(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<MockManagedReleaseRequest>,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    let mut leases = state
        .leases
        .lock()
        .expect("managed coordination lease store should not be poisoned");
    let Some(existing) = leases.get(&doc_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if existing.node_id.trim() != payload.node_id.trim()
        || existing.lease_id != Some(payload.lease_id)
        || existing.epoch != payload.epoch
    {
        return (StatusCode::CONFLICT, Json(existing.clone())).into_response();
    }

    leases.remove(&doc_id);
    StatusCode::NO_CONTENT.into_response()
}

async fn mock_managed_list_snapshots(
    State(state): State<MockManagedCoordinationServiceState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    let snapshots = state
        .snapshots
        .lock()
        .expect("managed snapshot store should not be poisoned");
    let documents = snapshots
        .values()
        .map(|snapshot| snapshot.document.clone())
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        Json(MockManagedSnapshotCatalogResponse { documents }),
    )
        .into_response()
}

async fn mock_managed_get_snapshot(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    match state.snapshot(&doc_id) {
        Some(snapshot) => (StatusCode::OK, Json(snapshot)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn mock_managed_put_snapshot(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<MockManagedSnapshotPayload>,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    if payload.document.id != doc_id {
        return StatusCode::BAD_REQUEST.into_response();
    }

    state
        .snapshots
        .lock()
        .expect("managed snapshot store should not be poisoned")
        .insert(doc_id, payload);
    StatusCode::NO_CONTENT.into_response()
}

async fn mock_managed_delete_snapshot(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    state
        .snapshots
        .lock()
        .expect("managed snapshot store should not be poisoned")
        .remove(&doc_id);
    StatusCode::NO_CONTENT.into_response()
}

impl RecordingRoomCoordinator {
    fn snapshot(&self) -> Vec<String> {
        self.events
            .lock()
            .expect("recording coordinator mutex should not be poisoned")
            .clone()
    }
}

impl RoomCoordinator for RecordingRoomCoordinator {
    fn mode(&self) -> &'static str {
        "recording"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        self.events
            .lock()
            .expect("recording coordinator mutex should not be poisoned")
            .push(format!("activate:{doc_id}"));
        Ok(())
    }

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        self.events
            .lock()
            .expect("recording coordinator mutex should not be poisoned")
            .push(format!("deactivate:{doc_id}"));
        Ok(())
    }
}

#[derive(Debug, Default)]
struct FailingRoomCoordinator;

impl RoomCoordinator for FailingRoomCoordinator {
    fn mode(&self) -> &'static str {
        "failing"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        Err(RoomCoordinatorError::Operation(format!(
            "unable to acquire lease for {doc_id}"
        )))
    }

    fn room_deactivated(&self, _doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        Ok(())
    }
}

fn decode_sync_message(payload: impl AsRef<[u8]>) -> SyncMessage {
    let message = Message::decode_v1(payload.as_ref()).expect("websocket payload should decode");
    match message {
        Message::Sync(message) => message,
        other => panic!("expected sync message, received {other:?}"),
    }
}

async fn wait_for_sqlite_room_lease_release(sqlite_path: &std::path::Path, doc_id: Uuid) {
    for _ in 0..20 {
        let connection = rusqlite::Connection::open(sqlite_path)
            .expect("sqlite coordinator file should open while waiting for release");
        let remaining: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM room_leases WHERE doc_id = ?1",
                [doc_id.to_string()],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if remaining == 0 {
            return;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    panic!("sqlite room lease for `{doc_id}` should be released after handoff");
}

async fn wait_for_managed_room_lease_release(
    state: &MockManagedCoordinationServiceState,
    doc_id: Uuid,
) {
    for _ in 0..100 {
        if state.lease(&doc_id).is_none() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    panic!("managed room lease for `{doc_id}` should be released after handoff");
}

async fn wait_for_managed_room_lease_owner(
    state: &MockManagedCoordinationServiceState,
    doc_id: Uuid,
    expected_node_id: &str,
) {
    for _ in 0..100 {
        if state
            .lease(&doc_id)
            .map(|lease| lease.node_id == expected_node_id)
            .unwrap_or(false)
        {
            return;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    panic!("managed room lease for `{doc_id}` should be owned by `{expected_node_id}`");
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
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("idle room eviction should succeed");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
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
    response.assert_header(
        "x-collab-owner-node-id",
        format!("node-for-{}", document.id),
    );
    response.assert_header("x-collab-owner-base-url", "http://node-b.internal:4000");
    response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-b.internal:4000/api/documents/{}", document.id),
    );
    response.assert_header(
        "location",
        format!("http://node-b.internal:4000/api/documents/{}", document.id),
    );

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
async fn document_detail_endpoint_rejects_non_local_file_room_owner() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("file-room-locator");
    config.room_locator = "file".to_owned();
    config.room_coordinator_state_dir = coordinator_dir.display().to_string();
    config.node_id = "node-a".to_owned();

    let state = AppState::from_config(&config).expect("state should initialize with file locator");
    let document = state
        .rooms()
        .create_document(Some("Remote file owner".to_owned()))
        .expect("document should be created");

    fs::write(
        coordinator_dir.join(format!("{}.json", document.id)),
        serde_json::to_vec(&serde_json::json!({
            "doc_id": document.id,
            "node_id": "node-b",
            "activated_at": "2026-04-20T00:00:00Z",
            "updated_at": "2026-04-20T00:00:00Z"
        }))
        .expect("file room state should serialize"),
    )
    .expect("file room state should be written");

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
    assert_eq!(payload["owner"]["node_id"], "node-b");
    assert!(payload["owner"]["base_url"].is_null());

    fs::remove_dir_all(coordinator_dir).expect("test state directory should be cleaned up");
}

#[tokio::test]
async fn document_detail_endpoint_includes_base_url_for_non_local_file_room_owner() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("file-room-locator-with-base-url");
    config.room_locator = "file".to_owned();
    config.room_coordinator_state_dir = coordinator_dir.display().to_string();
    config.node_id = "node-a".to_owned();

    let state = AppState::from_config(&config).expect("state should initialize with file locator");
    let document = state
        .rooms()
        .create_document(Some("Remote file owner with base url".to_owned()))
        .expect("document should be created");

    fs::write(
        coordinator_dir.join(format!("{}.json", document.id)),
        serde_json::to_vec(&serde_json::json!({
            "doc_id": document.id,
            "node_id": "node-b",
            "base_url": "http://node-b.internal:5001/",
            "activated_at": "2026-04-20T00:00:00Z",
            "updated_at": "2026-04-20T00:00:00Z"
        }))
        .expect("file room state should serialize"),
    )
    .expect("file room state should be written");

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
    assert_eq!(payload["owner"]["node_id"], "node-b");
    assert_eq!(payload["owner"]["base_url"], "http://node-b.internal:5001");

    fs::remove_dir_all(coordinator_dir).expect("test state directory should be cleaned up");
}

#[tokio::test]
async fn document_detail_endpoint_rejects_non_local_sqlite_room_owner() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("sqlite-room-locator");
    let sqlite_path = coordinator_dir.join("room-coordinator.sqlite3");
    config.room_locator = "sqlite".to_owned();
    config.room_coordinator_sqlite_path = sqlite_path.to_string_lossy().into_owned();
    config.node_id = "node-a".to_owned();

    let state =
        AppState::from_config(&config).expect("state should initialize with sqlite locator");
    let document = state
        .rooms()
        .create_document(Some("Remote sqlite owner".to_owned()))
        .expect("document should be created");

    let connection = rusqlite::Connection::open(&sqlite_path).expect("sqlite file should open");
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS room_leases (
                doc_id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                base_url TEXT,
                lease_id TEXT NOT NULL,
                epoch INTEGER NOT NULL,
                activated_at TEXT NOT NULL,
                renewed_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );",
        )
        .expect("sqlite schema should initialize");
    let now = Utc::now();
    connection
        .execute(
            "INSERT INTO room_leases (
                doc_id,
                node_id,
                base_url,
                lease_id,
                epoch,
                activated_at,
                renewed_at,
                expires_at
            ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                document.id.to_string(),
                "node-b",
                Uuid::new_v4().to_string(),
                2_i64,
                now.to_rfc3339(),
                now.to_rfc3339(),
                (now + ChronoDuration::seconds(30)).to_rfc3339(),
            ],
        )
        .expect("sqlite room lease should be written");

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
    assert_eq!(payload["owner"]["node_id"], "node-b");
    assert!(payload["owner"]["base_url"].is_null());

    fs::remove_dir_all(coordinator_dir).expect("test sqlite directory should be cleaned up");
}

#[tokio::test]
async fn document_detail_endpoint_includes_base_url_for_non_local_sqlite_room_owner() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("sqlite-room-locator-with-base-url");
    let sqlite_path = coordinator_dir.join("room-coordinator.sqlite3");
    config.room_locator = "sqlite".to_owned();
    config.room_coordinator_sqlite_path = sqlite_path.to_string_lossy().into_owned();
    config.node_id = "node-a".to_owned();

    let state =
        AppState::from_config(&config).expect("state should initialize with sqlite locator");
    let document = state
        .rooms()
        .create_document(Some("Remote sqlite owner with base url".to_owned()))
        .expect("document should be created");

    let connection = rusqlite::Connection::open(&sqlite_path).expect("sqlite file should open");
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS room_leases (
                doc_id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                base_url TEXT,
                lease_id TEXT NOT NULL,
                epoch INTEGER NOT NULL,
                activated_at TEXT NOT NULL,
                renewed_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );",
        )
        .expect("sqlite schema should initialize");
    let now = Utc::now();
    connection
        .execute(
            "INSERT INTO room_leases (
                doc_id,
                node_id,
                base_url,
                lease_id,
                epoch,
                activated_at,
                renewed_at,
                expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                document.id.to_string(),
                "node-b",
                "http://node-b.internal:5100/",
                Uuid::new_v4().to_string(),
                3_i64,
                now.to_rfc3339(),
                now.to_rfc3339(),
                (now + ChronoDuration::seconds(30)).to_rfc3339(),
            ],
        )
        .expect("sqlite room lease should be written");

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
    response.assert_header("x-collab-owner-node-id", "node-b");
    response.assert_header("x-collab-owner-base-url", "http://node-b.internal:5100");
    response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-b.internal:5100/api/documents/{}", document.id),
    );
    response.assert_header(
        "location",
        format!("http://node-b.internal:5100/api/documents/{}", document.id),
    );

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(payload["owner"]["node_id"], "node-b");
    assert_eq!(payload["owner"]["base_url"], "http://node-b.internal:5100");

    fs::remove_dir_all(coordinator_dir).expect("test sqlite directory should be cleaned up");
}

#[tokio::test]
async fn document_detail_endpoint_allows_expired_file_room_owner_state() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("file-room-locator-expired");
    config.room_locator = "file".to_owned();
    config.room_coordinator_state_dir = coordinator_dir.display().to_string();
    config.node_id = "node-a".to_owned();

    let state = AppState::from_config(&config).expect("state should initialize with file locator");
    let document = state
        .rooms()
        .create_document(Some("Expired remote file owner".to_owned()))
        .expect("document should be created");

    fs::write(
        coordinator_dir.join(format!("{}.json", document.id)),
        serde_json::to_vec(&serde_json::json!({
            "doc_id": document.id,
            "node_id": "node-b",
            "lease_id": Uuid::new_v4(),
            "epoch": 2,
            "activated_at": (Utc::now() - ChronoDuration::seconds(5)).to_rfc3339(),
            "renewed_at": (Utc::now() - ChronoDuration::seconds(4)).to_rfc3339(),
            "expires_at": (Utc::now() - ChronoDuration::seconds(1)).to_rfc3339()
        }))
        .expect("file room state should serialize"),
    )
    .expect("file room state should be written");

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    assert_eq!(payload["document"]["id"], document.id.to_string());

    fs::remove_dir_all(coordinator_dir).expect("test state directory should be cleaned up");
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
async fn websocket_room_coordinator_tracks_first_and_last_session() {
    let config = test_config();
    let coordinator = Arc::new(RecordingRoomCoordinator::default());
    let state = AppState::with_snapshot_store_locator_and_coordinator(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        Arc::new(InMemorySnapshotStore::new()),
        Arc::new(backend::collab::locator::LocalRoomLocator),
        coordinator.clone(),
    )
    .expect("state should initialize with recording coordinator");
    let document = state
        .rooms()
        .create_document(Some("Tracked room".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let websocket_a = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(
        coordinator.snapshot(),
        vec![format!("activate:{}", document.id)]
    );

    let websocket_b = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(
        coordinator.snapshot(),
        vec![format!("activate:{}", document.id)]
    );

    websocket_a.close().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(
        coordinator.snapshot(),
        vec![format!("activate:{}", document.id)]
    );

    websocket_b.close().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(
        coordinator.snapshot(),
        vec![
            format!("activate:{}", document.id),
            format!("deactivate:{}", document.id),
        ]
    );
}

#[tokio::test]
async fn websocket_room_activation_failure_does_not_leak_active_sessions() {
    let config = test_config();
    let state = AppState::with_snapshot_store_locator_and_coordinator(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        Arc::new(InMemorySnapshotStore::new()),
        Arc::new(backend::collab::locator::LocalRoomLocator),
        Arc::new(FailingRoomCoordinator),
    )
    .expect("state should initialize with failing coordinator");
    let document = state
        .rooms()
        .create_document(Some("Failed coordinator activation".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state.clone()).expect("app should build");
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

    tokio::time::sleep(Duration::from_millis(50)).await;
    websocket.close().await;

    let room = state
        .rooms()
        .get_or_restore(&document.id)
        .expect("room lookup should succeed")
        .expect("room should remain recoverable after activation failure");
    assert_eq!(room.active_sessions(), 0);
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
async fn websocket_endpoint_rejects_non_local_owner_with_redirect_headers() {
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
        .create_document(Some("Remote websocket owner".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let response = server
        .get_websocket(&format!("/ws/{}?source=edge", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status(StatusCode::CONFLICT);
    response.assert_header(
        "x-collab-owner-node-id",
        format!("node-for-{}", document.id),
    );
    response.assert_header("x-collab-owner-base-url", "http://node-b.internal:4000");
    response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-b.internal:4000/ws/{}?source=edge", document.id),
    );
    response.assert_header(
        "location",
        format!("http://node-b.internal:4000/ws/{}?source=edge", document.id),
    );

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
async fn websocket_endpoint_restores_latest_sqlite_snapshot_after_owner_handoff() {
    let shared_root = temp_snapshot_dir("sqlite-owner-handoff");

    let mut node_a_config = test_config();
    configure_shared_sqlite_collaboration(
        &mut node_a_config,
        &shared_root,
        "node-a",
        "http://node-a.internal:4300/",
    );

    let mut node_b_config = test_config();
    configure_shared_sqlite_collaboration(
        &mut node_b_config,
        &shared_root,
        "node-b",
        "http://node-b.internal:4301/",
    );

    let node_a_state =
        AppState::from_config(&node_a_config).expect("node-a state should initialize");
    let node_a_app = build_app(&node_a_config, node_a_state).expect("node-a app should build");
    let node_a_server = TestServer::builder().http_transport().build(node_a_app);

    let create_response = node_a_server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&node_a_config).as_str())
        .json(&serde_json::json!({
            "title": "Handoff document"
        }))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let doc_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();
    let doc_uuid = Uuid::parse_str(&doc_id).expect("created document id should be a UUID");
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned")
        .to_owned();

    let mut node_a_client = node_a_server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", node_a_config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await
        .into_websocket()
        .await;

    node_a_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;
    let initial_sync = decode_sync_message(node_a_client.receive_bytes().await);
    let SyncMessage::SyncStep2(initial_update) = initial_sync else {
        panic!("expected SyncStep2 during initial node-a handshake");
    };

    let node_a_doc = Doc::new();
    let node_a_text = node_a_doc.get_or_insert_text("content");
    let mut node_a_txn = node_a_doc.transact_mut();
    node_a_txn.apply_update(
        Update::decode_v1(initial_update.as_slice()).expect("initial sync payload should decode"),
    );
    node_a_text.insert(&mut node_a_txn, 0, "hello handoff");
    let client_update = node_a_txn.encode_update_v1();
    drop(node_a_txn);

    node_a_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::Update(client_update))
                .encode_v1()
                .into(),
        ))
        .await;

    let node_b_state =
        AppState::from_config(&node_b_config).expect("node-b state should initialize");
    assert!(
        node_b_state.rooms().get(&doc_uuid).is_none(),
        "distributed sqlite mode should not eagerly hydrate rooms on startup"
    );

    let node_b_app =
        build_app(&node_b_config, node_b_state.clone()).expect("node-b app should build");
    let node_b_server = TestServer::builder().http_transport().build(node_b_app);

    let standby_response = node_b_server
        .get(&format!("/api/documents/{doc_id}?probe=standby"))
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await;
    standby_response.assert_status(StatusCode::CONFLICT);
    standby_response.assert_header("x-collab-owner-node-id", "node-a");
    standby_response.assert_header("x-collab-owner-base-url", "http://node-a.internal:4300");
    standby_response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-a.internal:4300/api/documents/{doc_id}?probe=standby"),
    );

    node_a_client.close().await;
    let lease_path = shared_root.join("room-coordinator.sqlite3");
    wait_for_sqlite_room_lease_release(&lease_path, doc_uuid).await;

    let detail_response = {
        let mut last_status = None;
        let mut response = None;

        for _ in 0..100 {
            let next_response = node_b_server
                .get(&format!("/api/documents/{doc_id}"))
                .add_header(
                    "Authorization",
                    document_auth_header(&access_token).as_str(),
                )
                .await;
            let status = next_response.status_code();
            if status == StatusCode::OK {
                response = Some(next_response);
                break;
            }

            last_status = Some(status);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        response.unwrap_or_else(|| {
            panic!(
                "node-b detail restore should become available after managed handoff, last status was {:?}",
                last_status
            )
        })
    };
    detail_response.assert_status_ok();

    let mut node_b_client = node_b_server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", node_b_config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await
        .into_websocket()
        .await;

    node_b_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;
    let handoff_sync = decode_sync_message(node_b_client.receive_bytes().await);
    let SyncMessage::SyncStep2(handoff_update) = handoff_sync else {
        panic!("expected SyncStep2 during node-b handoff handshake");
    };

    let node_b_doc = Doc::new();
    let node_b_text = node_b_doc.get_or_insert_text("content");
    let mut node_b_txn = node_b_doc.transact_mut();
    node_b_txn.apply_update(
        Update::decode_v1(handoff_update.as_slice()).expect("handoff sync payload should decode"),
    );
    drop(node_b_txn);

    assert_eq!(
        node_b_text.get_string(&node_b_doc.transact()),
        "hello handoff"
    );

    node_b_client.close().await;
    fs::remove_dir_all(shared_root).expect("shared sqlite handoff directory should be cleaned up");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn document_detail_restores_latest_sqlite_snapshot_after_managed_owner_handoff() {
    let shared_root = temp_snapshot_dir("managed-owner-handoff");
    let harness = spawn_mock_managed_coordination_service(Some("managed-secret")).await;

    let mut node_a_config = test_config();
    configure_managed_coordination_with_shared_sqlite_snapshots(
        &mut node_a_config,
        &shared_root,
        "node-a",
        "http://node-a.internal:4300/",
        harness.base_url.as_str(),
        Some("managed-secret"),
    );

    let mut node_b_config = test_config();
    configure_managed_coordination_with_shared_sqlite_snapshots(
        &mut node_b_config,
        &shared_root,
        "node-b",
        "http://node-b.internal:4301/",
        harness.base_url.as_str(),
        Some("managed-secret"),
    );

    let node_a_state =
        AppState::from_config(&node_a_config).expect("node-a state should initialize");
    let node_a_app = build_app(&node_a_config, node_a_state).expect("node-a app should build");
    let node_a_server = TestServer::builder().http_transport().build(node_a_app);

    let create_response = node_a_server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&node_a_config).as_str())
        .json(&serde_json::json!({
            "title": "Managed handoff document"
        }))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let doc_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();
    let doc_uuid = Uuid::parse_str(&doc_id).expect("created document id should be a UUID");
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned")
        .to_owned();

    let mut node_a_client = node_a_server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", node_a_config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await
        .into_websocket()
        .await;

    node_a_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;
    let initial_sync = decode_sync_message(node_a_client.receive_bytes().await);
    let SyncMessage::SyncStep2(initial_update) = initial_sync else {
        panic!("expected SyncStep2 during initial node-a handshake");
    };

    let node_a_doc = Doc::new();
    let node_a_text = node_a_doc.get_or_insert_text("content");
    let mut node_a_txn = node_a_doc.transact_mut();
    node_a_txn.apply_update(
        Update::decode_v1(initial_update.as_slice()).expect("initial sync payload should decode"),
    );
    node_a_text.insert(&mut node_a_txn, 0, "hello managed handoff");
    let client_update = node_a_txn.encode_update_v1();
    drop(node_a_txn);

    node_a_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::Update(client_update))
                .encode_v1()
                .into(),
        ))
        .await;

    wait_for_managed_room_lease_owner(&harness.state, doc_uuid, "node-a").await;

    let node_b_state =
        AppState::from_config(&node_b_config).expect("node-b state should initialize");
    assert!(
        node_b_state.rooms().get(&doc_uuid).is_none(),
        "distributed managed mode should not eagerly hydrate rooms on startup"
    );

    let node_b_app =
        build_app(&node_b_config, node_b_state.clone()).expect("node-b app should build");
    let node_b_server = TestServer::builder().http_transport().build(node_b_app);

    let standby_response = node_b_server
        .get(&format!("/api/documents/{doc_id}?probe=managed-standby"))
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await;
    standby_response.assert_status(StatusCode::CONFLICT);
    standby_response.assert_header("x-collab-owner-node-id", "node-a");
    standby_response.assert_header("x-collab-owner-base-url", "http://node-a.internal:4300");
    standby_response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-a.internal:4300/api/documents/{doc_id}?probe=managed-standby"),
    );

    node_a_client.close().await;
    wait_for_managed_room_lease_release(&harness.state, doc_uuid).await;

    let detail_response = {
        let mut detail_response = None;
        let mut last_status = None;

        for _ in 0..100 {
            let response = node_b_server
                .get(&format!("/api/documents/{doc_id}"))
                .add_header(
                    "Authorization",
                    document_auth_header(&access_token).as_str(),
                )
                .await;
            let status = response.status_code();
            if status == StatusCode::OK {
                detail_response = Some(response);
                break;
            }

            last_status = Some(status);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        detail_response.unwrap_or_else(|| {
            panic!(
                "node-b detail restore should become available after managed handoff, last status was {:?}",
                last_status
            )
        })
    };
    detail_response.assert_status_ok();

    let restored_room = node_b_state
        .rooms()
        .get(&doc_uuid)
        .expect("detail restore should hydrate the room on node-b");
    let node_b_doc = Doc::new();
    let node_b_text = node_b_doc.get_or_insert_text("content");
    let restored_snapshot = restored_room
        .snapshot()
        .expect("restored room should snapshot after managed handoff");
    let mut restored_txn = node_b_doc.transact_mut();
    restored_txn.apply_update(
        Update::decode_v1(restored_snapshot.update.as_slice())
            .expect("managed handoff snapshot should decode"),
    );
    drop(restored_txn);

    assert_eq!(
        node_b_text.get_string(&node_b_doc.transact()),
        "hello managed handoff"
    );

    fs::remove_dir_all(shared_root).expect("managed handoff directory should be cleaned up");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn app_state_restores_latest_managed_snapshot_after_managed_owner_handoff() {
    let harness = spawn_mock_managed_coordination_service(Some("managed-secret")).await;

    let mut node_a_config = test_config();
    configure_managed_coordination_with_managed_snapshots(
        &mut node_a_config,
        "node-a",
        "http://node-a.internal:4300/",
        harness.base_url.as_str(),
        harness.snapshot_base_url.as_str(),
        Some("managed-secret"),
    );

    let mut node_b_config = test_config();
    configure_managed_coordination_with_managed_snapshots(
        &mut node_b_config,
        "node-b",
        "http://node-b.internal:4301/",
        harness.base_url.as_str(),
        harness.snapshot_base_url.as_str(),
        Some("managed-secret"),
    );

    let node_a_state =
        AppState::from_config(&node_a_config).expect("node-a state should initialize");
    let document = node_a_state
        .rooms()
        .create_document(Some("Managed durability handoff document".to_owned()))
        .expect("document should be created");
    let doc_uuid = document.id;
    let node_a_room = node_a_state
        .rooms()
        .get(&doc_uuid)
        .expect("created document should have an active room");
    {
        let node_a_doc = node_a_room.awareness().write().await.doc().clone();
        let node_a_text = node_a_doc.get_or_insert_text("content");
        let mut node_a_txn = node_a_doc.transact_mut();
        node_a_text.insert(&mut node_a_txn, 0, "hello managed durability handoff");
    }

    assert_eq!(node_a_room.start_session(), 1);
    node_a_state
        .room_coordinator()
        .room_activated(&doc_uuid)
        .expect("node-a should acquire the managed lease");

    wait_for_managed_room_lease_owner(&harness.state, doc_uuid, "node-a").await;

    let node_b_state =
        AppState::from_config(&node_b_config).expect("node-b state should initialize");
    assert!(
        node_b_state.rooms().get(&doc_uuid).is_none(),
        "distributed managed mode should not eagerly hydrate rooms on startup"
    );
    let listed_documents = node_b_state
        .rooms()
        .list_documents()
        .expect("managed snapshot catalog should load while the room stays cold");
    assert_eq!(listed_documents.len(), 1);
    assert_eq!(listed_documents[0].id, doc_uuid);

    let error = node_b_state
        .ensure_local_room_owner(&doc_uuid)
        .expect_err("node-b should observe node-a as the active managed owner");
    match error {
        AppError::RemoteOwner {
            owner_node_id,
            owner_base_url,
            ..
        } => {
            assert_eq!(owner_node_id, "node-a");
            assert_eq!(
                owner_base_url.as_deref(),
                Some("http://node-a.internal:4300")
            );
        }
        other => panic!("expected remote owner error, received {other:?}"),
    }

    let teardown = node_a_state
        .rooms()
        .persist_and_evict_if_idle(&doc_uuid, &node_a_room)
        .expect("node-a should persist the managed snapshot before handoff");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
    node_a_state
        .room_coordinator()
        .room_deactivated(&doc_uuid)
        .expect("node-a should release the managed lease after persisting");
    wait_for_managed_room_lease_release(&harness.state, doc_uuid).await;

    node_b_state
        .ensure_local_room_owner(&doc_uuid)
        .expect("node-b should resolve locally after the managed lease is released");

    let restored_room = node_b_state
        .rooms()
        .get_or_restore(&doc_uuid)
        .expect("node-b restore should query the managed snapshot store")
        .expect("managed snapshot should restore after owner handoff");
    let node_b_doc = Doc::new();
    let node_b_text = node_b_doc.get_or_insert_text("content");
    let restored_snapshot = restored_room
        .snapshot()
        .expect("restored room should snapshot after managed-managed handoff");
    let mut node_b_txn = node_b_doc.transact_mut();
    node_b_txn.apply_update(
        Update::decode_v1(restored_snapshot.update.as_slice())
            .expect("managed-managed handoff snapshot should decode"),
    );
    drop(node_b_txn);

    assert_eq!(
        node_b_text.get_string(&node_b_doc.transact()),
        "hello managed durability handoff"
    );
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
    let teardown = bootstrap_registry
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("idle room eviction should succeed");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
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
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to disk on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

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
async fn app_state_uses_sqlite_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("sqlite-store-config");
    let snapshot_path = snapshot_dir.join("snapshots.sqlite3");
    config.snapshot_store = "sqlite".to_owned();
    config.snapshot_sqlite_path = snapshot_path.to_string_lossy().into_owned();

    let state = AppState::from_config(&config).expect("state should initialize with sqlite store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to sqlite".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to sqlite on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted sqlite snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_jammdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("jammdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jammdb");
    configure_jammdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with jammdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to jammdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to jammdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted jammdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_heed_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("heed-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.heed");
    configure_heed_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with heed store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to heed".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to heed on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted heed snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_fjall_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("fjall-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.fjall");
    configure_fjall_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with fjall store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to fjall".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to fjall on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted fjall snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_persy_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("persy-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.persy");
    configure_persy_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with persy store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to persy".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to persy on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted persy snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_rejects_managed_snapshot_store_without_base_url() {
    let mut config = test_config();
    config.snapshot_store = "managed".to_owned();

    let error = match AppState::from_config(&config) {
        Ok(_) => panic!("managed snapshot store should require base url"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("SNAPSHOT_MANAGED_BASE_URL is required when SNAPSHOT_STORE=managed"),
        "unexpected error: {error}"
    );
}

#[test]
fn app_state_uses_redb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("redb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.redb");
    configure_redb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with redb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to redb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to redb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted redb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_sled_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("sled-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.sled");
    configure_sled_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with sled store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to sled".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to sled on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted sled snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_rejects_s3_snapshot_store_without_endpoint() {
    let mut config = test_config();
    config.snapshot_store = "s3".to_owned();

    let error = match AppState::from_config(&config) {
        Ok(_) => panic!("s3 snapshot store should require endpoint"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("SNAPSHOT_S3_ENDPOINT is required when SNAPSHOT_STORE=s3"),
        "unexpected error: {error}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_state_uses_managed_snapshot_store_from_config() {
    let harness = spawn_mock_managed_coordination_service(Some("snapshot-secret")).await;

    let mut config = test_config();
    configure_managed_snapshot_store(
        &mut config,
        &harness.snapshot_base_url,
        Some("snapshot-secret"),
    );

    let state = AppState::from_config(&config)
        .expect("state should initialize with managed snapshot store");
    let document = state
        .rooms()
        .create_document(Some("Persisted to managed store".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to managed store on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    let persisted_snapshot = harness
        .state
        .snapshot(&document.id)
        .expect("managed snapshot service should store the snapshot");
    assert_eq!(persisted_snapshot.document.id, document.id);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted managed snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_state_uses_s3_snapshot_store_from_config() {
    let harness = spawn_mock_s3_snapshot_service().await;

    let mut config = test_config();
    configure_s3_snapshot_store(
        &mut config,
        &harness.endpoint,
        &harness.bucket,
        &harness.access_key_id,
        &harness.secret_access_key,
    );

    let state =
        AppState::from_config(&config).expect("state should initialize with s3 snapshot store");
    let document = state
        .rooms()
        .create_document(Some("Persisted to s3".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to s3 on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    let object_key = format!("snapshots/test-suite/{}.json", document.id);
    let persisted_snapshot = harness
        .state
        .object(&object_key)
        .expect("mock s3 service should store the snapshot object");
    assert!(!persisted_snapshot.is_empty());

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted s3 snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(
        harness
            .state
            .last_authorization()
            .is_some_and(|header| header.contains("Credential=test-access-key/")),
        "s3 requests should be signed with the configured access key"
    );
}

#[tokio::test]
async fn app_state_skips_startup_room_hydration_in_distributed_sqlite_mode() {
    let shared_root = temp_snapshot_dir("sqlite-distributed-skip-hydrate");

    let mut writer_config = test_config();
    configure_shared_sqlite_collaboration(
        &mut writer_config,
        &shared_root,
        "node-a",
        "http://node-a.internal:4400/",
    );
    let writer_state =
        AppState::from_config(&writer_config).expect("writer state should initialize");
    let document = writer_state
        .rooms()
        .create_document(Some("Distributed hydrate guard".to_owned()))
        .expect("document should be created");

    let mut reader_config = test_config();
    configure_shared_sqlite_collaboration(
        &mut reader_config,
        &shared_root,
        "node-b",
        "http://node-b.internal:4401/",
    );
    let reader_state =
        AppState::from_config(&reader_config).expect("reader state should initialize");

    assert!(
        reader_state.rooms().get(&document.id).is_none(),
        "distributed sqlite mode should leave rooms cold until ownership is checked"
    );
    let listed_documents = reader_state
        .rooms()
        .list_documents()
        .expect("document catalog should still load from shared snapshot store");
    assert_eq!(listed_documents.len(), 1);
    assert_eq!(listed_documents[0].id, document.id);
    assert_eq!(listed_documents[0].title, document.title);

    fs::remove_dir_all(shared_root).expect("shared sqlite test directory should be cleaned up");
}

#[tokio::test]
async fn app_state_uses_logging_room_coordinator_from_config() {
    let mut config = test_config();
    config.room_coordinator = "logging".to_owned();

    let state =
        AppState::from_config(&config).expect("state should initialize with logging coordinator");
    assert_eq!(state.room_coordinator().mode(), "logging");

    let document = state
        .rooms()
        .create_document(Some("Logged room".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("logging coordinator should not affect snapshot persistence");

    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
}

#[tokio::test]
async fn app_state_uses_file_room_coordinator_from_config() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("file-room-coordinator");
    config.room_coordinator = "file".to_owned();
    config.room_coordinator_state_dir = coordinator_dir.display().to_string();
    config.node_base_url = Some("http://node-a.internal:4100/".to_owned());

    let state =
        AppState::from_config(&config).expect("state should initialize with file coordinator");
    assert_eq!(state.room_coordinator().mode(), "file");

    let document = state
        .rooms()
        .create_document(Some("File coordinated room".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    state
        .room_coordinator()
        .room_activated(&document.id)
        .expect("file coordinator should persist active room state");
    let state_path = coordinator_dir.join(format!("{}.json", document.id));
    assert!(state_path.exists());

    let persisted_state: Value = serde_json::from_slice(
        &fs::read(&state_path).expect("file coordinator should persist active room state"),
    )
    .expect("file room coordinator state should deserialize");
    assert_eq!(persisted_state["base_url"], "http://node-a.internal:4100");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("file coordinator should not affect snapshot persistence");

    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    state
        .room_coordinator()
        .room_deactivated(&document.id)
        .expect("file coordinator should remove active room state");
    assert!(!state_path.exists());

    fs::remove_dir_all(coordinator_dir).expect("test coordinator directory should be cleaned up");
}

#[tokio::test]
async fn app_state_uses_sqlite_room_coordinator_from_config() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("sqlite-room-coordinator");
    let sqlite_path = coordinator_dir.join("room-coordinator.sqlite3");
    config.room_coordinator = "sqlite".to_owned();
    config.room_coordinator_sqlite_path = sqlite_path.to_string_lossy().into_owned();
    config.node_base_url = Some("http://node-a.internal:4200/".to_owned());

    let state =
        AppState::from_config(&config).expect("state should initialize with sqlite coordinator");
    assert_eq!(state.room_coordinator().mode(), "sqlite");

    let document = state
        .rooms()
        .create_document(Some("Sqlite coordinated room".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    state
        .room_coordinator()
        .room_activated(&document.id)
        .expect("sqlite coordinator should persist active room state");
    let connection = rusqlite::Connection::open(&sqlite_path).expect("sqlite file should open");
    let persisted_state: Value = connection
        .query_row(
            "SELECT json_object(
                'doc_id', doc_id,
                'node_id', node_id,
                'base_url', base_url,
                'lease_id', lease_id,
                'epoch', epoch,
                'activated_at', activated_at,
                'renewed_at', renewed_at,
                'expires_at', expires_at
            )
             FROM room_leases
             WHERE doc_id = ?1",
            [document.id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .map(|json| serde_json::from_str(&json).expect("sqlite room lease json should parse"))
        .expect("sqlite coordinator should persist active room state");
    assert_eq!(persisted_state["base_url"], "http://node-a.internal:4200");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("sqlite coordinator should not affect snapshot persistence");

    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    state
        .room_coordinator()
        .room_deactivated(&document.id)
        .expect("sqlite coordinator should remove active room state");
    let remaining: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM room_leases WHERE doc_id = ?1",
            [document.id.to_string()],
            |row| row.get(0),
        )
        .expect("sqlite coordinator should query room lease count");
    assert_eq!(remaining, 0);

    fs::remove_dir_all(coordinator_dir).expect("test coordinator directory should be cleaned up");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_state_uses_managed_room_coordination_from_config() {
    let harness = spawn_mock_managed_coordination_service(Some("managed-secret")).await;

    let mut writer_config = test_config();
    writer_config.room_locator = "managed".to_owned();
    writer_config.room_coordinator = "managed".to_owned();
    writer_config.room_coordination_managed_base_url = Some(harness.base_url.clone());
    writer_config.room_coordination_managed_auth_token = Some("managed-secret".to_owned());
    writer_config.room_coordinator_heartbeat_interval_secs = 1;
    writer_config.room_coordinator_lease_ttl_secs = 3;
    writer_config.node_id = "node-a".to_owned();
    writer_config.node_base_url = Some("http://node-a.internal:4300/".to_owned());

    let writer_state =
        AppState::from_config(&writer_config).expect("state should initialize with managed mode");
    assert_eq!(writer_state.room_coordinator().mode(), "managed");

    let document = writer_state
        .rooms()
        .create_document(Some("Managed coordinated room".to_owned()))
        .expect("document should be created");

    writer_state
        .room_coordinator()
        .room_activated(&document.id)
        .expect("managed coordinator should persist active room state");

    let initial_lease = harness
        .state
        .lease(&document.id)
        .expect("managed coordination service should store the acquired lease");
    assert_eq!(initial_lease.node_id, "node-a");
    assert_eq!(
        initial_lease.base_url,
        Some("http://node-a.internal:4300".to_owned())
    );

    tokio::time::sleep(Duration::from_millis(1_100)).await;

    let renewed_lease = harness
        .state
        .lease(&document.id)
        .expect("managed coordination service should keep the renewed lease");
    assert_eq!(renewed_lease.lease_id, initial_lease.lease_id);
    assert_eq!(renewed_lease.epoch, initial_lease.epoch);
    assert!(
        renewed_lease.renewed_at > initial_lease.renewed_at,
        "managed coordinator heartbeat should advance renewed_at"
    );

    let mut reader_config = test_config();
    reader_config.room_locator = "managed".to_owned();
    reader_config.room_coordination_managed_base_url = Some(harness.base_url.clone());
    reader_config.room_coordination_managed_auth_token = Some("managed-secret".to_owned());
    reader_config.node_id = "node-b".to_owned();

    let reader_state =
        AppState::from_config(&reader_config).expect("reader state should initialize");
    let error = reader_state
        .ensure_local_room_owner(&document.id)
        .expect_err("managed locator should report the remote owner while the lease is active");

    match error {
        AppError::RemoteOwner {
            owner_node_id,
            owner_base_url,
            ..
        } => {
            assert_eq!(owner_node_id, "node-a");
            assert_eq!(
                owner_base_url.as_deref(),
                Some("http://node-a.internal:4300")
            );
        }
        other => panic!("expected remote owner error, received {other:?}"),
    }

    writer_state
        .room_coordinator()
        .room_deactivated(&document.id)
        .expect("managed coordinator should release active room state");
    assert!(
        harness.state.lease(&document.id).is_none(),
        "managed coordination service should remove the lease after release"
    );
    reader_state
        .ensure_local_room_owner(&document.id)
        .expect("managed locator should resolve locally after lease release");
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
fn sqlite_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("sqlite-store-unit");
    let snapshot_path = snapshot_dir.join("snapshots.sqlite3");
    let store = SqliteSnapshotStore::new(&snapshot_path).expect("sqlite store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Sqlite".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to sqlite store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from sqlite store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from sqlite store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn jammdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("jammdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jammdb");
    let store =
        JammdbSnapshotStore::new(&snapshot_path).expect("jammdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Jammdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to jammdb store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from jammdb store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from jammdb store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn heed_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("heed-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.heed");
    let store =
        HeedSnapshotStore::new(&snapshot_path).expect("heed snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Heed".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to heed store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from heed store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from heed store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn fjall_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("fjall-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.fjall");
    let store =
        FjallSnapshotStore::new(&snapshot_path).expect("fjall snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Fjall".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to fjall store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from fjall store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from fjall store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn persy_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("persy-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.persy");
    let store =
        PersySnapshotStore::new(&snapshot_path).expect("persy snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Persy".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to persy store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from persy store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from persy store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn managed_snapshot_store_round_trips_document_catalog() {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime should initialize");
    let harness = runtime.block_on(spawn_mock_managed_coordination_service(Some(
        "snapshot-secret",
    )));
    let store = ManagedSnapshotStore::new(
        &harness.snapshot_base_url,
        Some("snapshot-secret".to_owned()),
        Duration::from_secs(5),
    )
    .expect("managed snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Managed".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to managed store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from managed store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from managed store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);
}

#[test]
fn redb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("redb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.redb");
    let store =
        RedbSnapshotStore::new(&snapshot_path).expect("redb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Redb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to redb store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from redb store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from redb store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn sled_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("sled-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.sled");
    let store =
        SledSnapshotStore::new(&snapshot_path).expect("sled snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Sled".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to sled store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from sled store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from sled store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn s3_snapshot_store_round_trips_document_catalog() {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime should initialize");
    let harness = runtime.block_on(spawn_mock_s3_snapshot_service());
    let store = S3SnapshotStore::new(
        &harness.endpoint,
        "us-east-1",
        &harness.bucket,
        "snapshots/unit-tests/",
        &harness.access_key_id,
        &harness.secret_access_key,
        None,
        Duration::from_secs(5),
        true,
    )
    .expect("s3 snapshot store should initialize");
    let document = backend::models::document::Document::new(Uuid::new_v4(), Some("S3".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to s3 store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from s3 store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from s3 store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);
    assert!(
        harness
            .state
            .last_authorization()
            .is_some_and(|header| header.contains("Credential=test-access-key/")),
        "s3 requests should be signed with the configured access key"
    );
}

#[test]
fn sqlite_snapshot_store_skips_corrupt_rows_when_listing_documents() {
    let snapshot_dir = temp_snapshot_dir("sqlite-store-corrupt-catalog");
    let snapshot_path = snapshot_dir.join("snapshots.sqlite3");
    let store = SqliteSnapshotStore::new(&snapshot_path).expect("sqlite store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Catalog".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![7, 8, 9]))
        .expect("valid snapshot should save");

    let corrupt_doc_id = Uuid::new_v4();
    let connection =
        rusqlite::Connection::open(&snapshot_path).expect("sqlite file should be writable");
    connection
        .execute(
            "INSERT INTO snapshots (doc_id, title, created_at, updated_at, access_token, update_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                corrupt_doc_id.to_string(),
                "Corrupt",
                "not-a-timestamp",
                "not-a-timestamp",
                "token",
                vec![1_u8, 2, 3]
            ],
        )
        .expect("corrupt sqlite snapshot row should be written");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should skip corrupt sqlite rows");
    let corrupt_snapshot_error = store
        .load_snapshot(&corrupt_doc_id)
        .expect_err("directly loading a corrupt sqlite snapshot should still fail");

    assert_eq!(listed_documents, vec![document]);
    assert!(matches!(
        corrupt_snapshot_error,
        backend::storage::StorageError::CorruptSnapshot(id) if id == corrupt_doc_id
    ));

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
