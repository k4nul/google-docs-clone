use std::{borrow::Cow, sync::Arc, time::Duration};

use axum::{
    extract::{
        OriginalUri, Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{
        HeaderMap, Uri,
        header::{AUTHORIZATION, ORIGIN},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, Stream, StreamExt, stream::SplitStream};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{info, warn};
use uuid::Uuid;
use yrs::{
    sync::{Error as SyncError, Message as SyncProtocolMessage},
    updates::encoder::Encode,
};
use yrs_axum::ws::AxumSink;

use crate::{
    auth::{authorize_document_token, require_bearer_token},
    collab::coordinator::RoomCoordinator,
    collab::protocol::ValidatingProtocol,
    collab::rooms::{Room, RoomRegistry},
    errors::{AppError, AppResult},
    http_params::parse_uuid_param,
    state::AppState,
};

const WEBSOCKET_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

pub async fn ws_handler(
    Path(raw_doc_id): Path<String>,
    OriginalUri(original_uri): OriginalUri,
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> AppResult<impl IntoResponse> {
    let (doc_id, room) = resolve_websocket_room(&state, &headers, &original_uri, &raw_doc_id)?;

    let registry = state.rooms_registry();
    let coordinator = state.room_coordinator();
    Ok(ws.on_upgrade(move |socket| serve_room_socket(socket, registry, coordinator, room, doc_id)))
}

async fn serve_room_socket(
    socket: WebSocket,
    registry: Arc<RoomRegistry>,
    coordinator: Arc<dyn RoomCoordinator>,
    room: Arc<Room>,
    doc_id: Uuid,
) {
    let active_sessions = room.start_session();
    info!(
        %doc_id,
        active_sessions,
        coordinator_mode = coordinator.mode(),
        "websocket collaboration session started"
    );

    if active_sessions == 1
        && let Err(error) = coordinator.room_activated(&doc_id)
    {
        warn!(
            %doc_id,
            coordinator_mode = coordinator.mode(),
            %error,
            "failed to activate room coordinator for websocket collaboration session"
        );

        match registry.persist_and_evict_if_idle(&doc_id, &room) {
            Ok(teardown) => info!(
                %doc_id,
                remaining_sessions = teardown.remaining_sessions,
                evicted = teardown.evicted,
                "rolled back websocket session after coordinator activation failure"
            ),
            Err(cleanup_error) => warn!(
                %doc_id,
                %cleanup_error,
                "failed to roll back websocket session after coordinator activation failure"
            ),
        }

        return;
    }

    let broadcast_group = room.broadcast_group().await;
    let (sink, stream) = socket.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = BinaryAxumStream::from(stream);
    let heartbeat = spawn_socket_heartbeat(sink.clone(), doc_id);
    let subscription = broadcast_group.subscribe_with(sink, stream, ValidatingProtocol);

    match subscription.completed().await {
        Ok(()) => info!(%doc_id, "websocket collaboration session ended"),
        Err(error) => warn!(%doc_id, %error, "websocket collaboration session failed"),
    }
    heartbeat.abort();

    match registry.persist_and_evict_if_idle(&doc_id, &room) {
        Ok(teardown) if teardown.evicted => {
            info!(%doc_id, "persisted snapshot and evicted idle room");
            if let Err(error) = coordinator.room_deactivated(&doc_id) {
                warn!(
                    %doc_id,
                    coordinator_mode = coordinator.mode(),
                    %error,
                    "failed to deactivate room coordinator after websocket session"
                );
            }
        }
        Ok(teardown) => info!(
            %doc_id,
            active_sessions = teardown.remaining_sessions,
            "room remains active after websocket session"
        ),
        Err(error) => warn!(%doc_id, %error, "failed to persist snapshot after websocket session"),
    }
}

fn spawn_socket_heartbeat(sink: Arc<Mutex<AxumSink>>, doc_id: Uuid) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(WEBSOCKET_HEARTBEAT_INTERVAL).await;

            let mut sink = sink.lock().await;
            if let Err(error) = sink.send(websocket_heartbeat_payload()).await {
                warn!(
                    %doc_id,
                    %error,
                    "failed to send websocket heartbeat"
                );
                return;
            }
        }
    })
}

fn websocket_heartbeat_payload() -> Vec<u8> {
    SyncProtocolMessage::AwarenessQuery.encode_v1()
}

#[derive(Debug, PartialEq, Eq)]
enum SocketMessagePayload {
    Binary(Vec<u8>),
    Ignore,
    Close,
}

#[derive(Debug)]
struct BinaryAxumStream(SplitStream<WebSocket>);

impl From<SplitStream<WebSocket>> for BinaryAxumStream {
    fn from(stream: SplitStream<WebSocket>) -> Self {
        Self(stream)
    }
}

impl Stream for BinaryAxumStream {
    type Item = Result<Vec<u8>, SyncError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.0).poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(Ok(message))) => match socket_message_payload(message) {
                    SocketMessagePayload::Binary(payload) => {
                        return Poll::Ready(Some(Ok(payload)));
                    }
                    SocketMessagePayload::Ignore => continue,
                    SocketMessagePayload::Close => return Poll::Ready(None),
                },
                Poll::Ready(Some(Err(error))) => {
                    return Poll::Ready(Some(Err(SyncError::Other(error.into()))));
                }
            }
        }
    }
}

fn socket_message_payload(message: Message) -> SocketMessagePayload {
    match message {
        Message::Binary(payload) => SocketMessagePayload::Binary(payload.to_vec()),
        Message::Close(_) => SocketMessagePayload::Close,
        Message::Text(_) | Message::Ping(_) | Message::Pong(_) => SocketMessagePayload::Ignore,
    }
}

fn resolve_websocket_room(
    state: &AppState,
    headers: &HeaderMap,
    request_uri: &Uri,
    raw_doc_id: &str,
) -> AppResult<(Uuid, Arc<Room>)> {
    let doc_id = parse_uuid_param("doc_id", raw_doc_id)?;
    validate_origin(state, headers, doc_id)?;
    state.ensure_local_room_owner_for_request(&doc_id, request_uri)?;
    let room = state
        .rooms()
        .get_or_restore(&doc_id)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("document `{doc_id}` was not found")))?;
    let token = require_websocket_access_token(headers, request_uri)?;
    authorize_document_token(token.as_ref(), &room, doc_id)?;

    Ok((doc_id, room))
}

fn require_websocket_access_token<'a>(
    headers: &'a HeaderMap,
    request_uri: &Uri,
) -> AppResult<Cow<'a, str>> {
    if headers.contains_key(AUTHORIZATION) {
        return require_bearer_token(headers).map(Cow::Borrowed);
    }

    request_uri
        .query()
        .and_then(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .find(|(name, _)| name == "access_token")
                .map(|(_, value)| value.trim().to_owned())
        })
        .filter(|token| !token.is_empty())
        .map(Cow::Owned)
        .ok_or_else(|| {
            AppError::Unauthorized(
                "Authorization header or access_token query parameter is required".to_owned(),
            )
        })
}

fn validate_origin(state: &AppState, headers: &HeaderMap, doc_id: Uuid) -> AppResult<()> {
    let Some(origin) = headers.get(ORIGIN) else {
        warn!(%doc_id, "rejected websocket connection without origin header");
        return Err(AppError::Forbidden(
            "Origin header is required for websocket connections".to_owned(),
        ));
    };

    if !state.frontend_origin_allowed(origin) {
        let received_origin = origin.to_str().unwrap_or("<invalid origin header>");
        warn!(
            %doc_id,
            received_origin,
            allowed_origins = state.frontend_origin(),
            "rejected websocket connection with disallowed origin"
        );
        return Err(AppError::Forbidden(format!(
            "Origin `{received_origin}` is not allowed for websocket connections"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    use crate::{
        collab::locator::{ResolvedRoom, RoomLocator, RoomLocatorError, RoomOwnerHint},
        config::DEFAULT_FRONTEND_ORIGIN,
        storage::InMemorySnapshotStore,
    };
    use yrs::{sync::Message as SyncProtocolMessage, updates::decoder::Decode};

    #[test]
    fn websocket_heartbeat_uses_valid_awareness_query_message() {
        let message = SyncProtocolMessage::decode_v1(websocket_heartbeat_payload().as_slice())
            .expect("heartbeat should decode as a Yjs protocol message");

        assert_eq!(message, SyncProtocolMessage::AwarenessQuery);
    }

    #[test]
    fn websocket_stream_forwards_only_binary_sync_payloads() {
        assert_eq!(
            socket_message_payload(Message::Binary(vec![0, 1, 2].into())),
            SocketMessagePayload::Binary(vec![0, 1, 2])
        );
    }

    #[test]
    fn websocket_stream_ignores_non_sync_control_frames() {
        assert_eq!(
            socket_message_payload(Message::Ping(Vec::new().into())),
            SocketMessagePayload::Ignore
        );
        assert_eq!(
            socket_message_payload(Message::Pong(Vec::new().into())),
            SocketMessagePayload::Ignore
        );
        assert_eq!(
            socket_message_payload(Message::Text("ignored".into())),
            SocketMessagePayload::Ignore
        );
    }

    #[test]
    fn websocket_stream_treats_close_as_normal_end() {
        assert_eq!(
            socket_message_payload(Message::Close(None)),
            SocketMessagePayload::Close
        );
    }

    #[test]
    fn websocket_room_resolution_accepts_any_origin_wildcard() {
        let state = AppState::new(
            crate::config::FRONTEND_ORIGIN_WILDCARD,
            crate::config::DEFAULT_API_TOKEN,
        );
        let document = state
            .rooms()
            .create_document(Some("Wildcard websocket origin".to_owned()))
            .expect("document should be created");

        let mut headers = HeaderMap::new();
        headers.insert(
            ORIGIN,
            HeaderValue::from_static("http://new-domain.test:5173"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", document.access_token()))
                .expect("document authorization header should be valid"),
        );
        let request_uri: Uri = format!("/ws/{}", document.id)
            .parse()
            .expect("websocket request URI should parse");

        resolve_websocket_room(&state, &headers, &request_uri, &document.id.to_string())
            .expect("wildcard frontend origin should allow any browser origin");
    }

    #[test]
    fn websocket_room_resolution_accepts_comma_separated_origins() {
        let state = AppState::new(
            "http://one.test:5173, http://two.test:5173",
            crate::config::DEFAULT_API_TOKEN,
        );
        let document = state
            .rooms()
            .create_document(Some("Listed websocket origin".to_owned()))
            .expect("document should be created");
        let request_uri: Uri = format!("/ws/{}", document.id)
            .parse()
            .expect("websocket request URI should parse");

        let mut allowed_headers = HeaderMap::new();
        allowed_headers.insert(ORIGIN, HeaderValue::from_static("http://two.test:5173"));
        allowed_headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", document.access_token()))
                .expect("document authorization header should be valid"),
        );
        resolve_websocket_room(
            &state,
            &allowed_headers,
            &request_uri,
            &document.id.to_string(),
        )
        .expect("listed frontend origin should be accepted");

        let mut rejected_headers = HeaderMap::new();
        rejected_headers.insert(ORIGIN, HeaderValue::from_static("http://three.test:5173"));
        rejected_headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", document.access_token()))
                .expect("document authorization header should be valid"),
        );
        let error = match resolve_websocket_room(
            &state,
            &rejected_headers,
            &request_uri,
            &document.id.to_string(),
        ) {
            Ok(_) => panic!("unlisted frontend origin should be rejected"),
            Err(error) => error,
        };

        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[test]
    fn websocket_room_resolution_accepts_document_token_query_parameter() {
        let state = AppState::new(
            crate::config::FRONTEND_ORIGIN_WILDCARD,
            crate::config::DEFAULT_API_TOKEN,
        );
        let document = state
            .rooms()
            .create_document(Some("Query token websocket".to_owned()))
            .expect("document should be created");

        let mut headers = HeaderMap::new();
        headers.insert(ORIGIN, HeaderValue::from_static("http://docs.test:5173"));
        let request_uri: Uri = format!(
            "/ws/{}?access_token={}",
            document.id,
            document.access_token()
        )
        .parse()
        .expect("websocket request URI should parse");

        resolve_websocket_room(&state, &headers, &request_uri, &document.id.to_string())
            .expect("query access token should authorize websocket access");
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

    #[test]
    fn websocket_room_resolution_rejects_non_local_owner() {
        let state = AppState::with_snapshot_store_and_locator(
            DEFAULT_FRONTEND_ORIGIN,
            crate::config::DEFAULT_API_TOKEN,
            Arc::new(InMemorySnapshotStore::new()),
            Arc::new(RemoteRoomLocator),
        )
        .expect("state should initialize with rejecting locator");
        let document = state
            .rooms()
            .create_document(Some("Remote websocket owner".to_owned()))
            .expect("document should be created");

        let mut headers = HeaderMap::new();
        headers.insert(ORIGIN, HeaderValue::from_static(DEFAULT_FRONTEND_ORIGIN));

        let request_uri: Uri = format!("/ws/{}", document.id)
            .parse()
            .expect("websocket request URI should parse");
        let error = match resolve_websocket_room(
            &state,
            &headers,
            &request_uri,
            &document.id.to_string(),
        ) {
            Ok(_) => panic!("non-local owner should reject websocket room resolution"),
            Err(error) => error,
        };

        match error {
            AppError::RemoteOwner {
                message,
                owner_node_id,
                owner_base_url,
                redirect_url,
            } => {
                assert_eq!(
                    message,
                    format!(
                        "document `{}` is owned by another collaboration node",
                        document.id
                    )
                );
                assert_eq!(owner_node_id, format!("node-for-{}", document.id));
                assert_eq!(
                    owner_base_url,
                    Some("http://node-b.internal:4000".to_owned())
                );
                assert_eq!(
                    redirect_url,
                    Some(format!("http://node-b.internal:4000/ws/{}", document.id))
                );
            }
            other => panic!("expected conflict, received {other:?}"),
        }
    }

    #[test]
    fn websocket_room_resolution_strips_query_from_remote_owner_redirect() {
        let state = AppState::with_snapshot_store_and_locator(
            DEFAULT_FRONTEND_ORIGIN,
            crate::config::DEFAULT_API_TOKEN,
            Arc::new(InMemorySnapshotStore::new()),
            Arc::new(RemoteRoomLocator),
        )
        .expect("state should initialize with rejecting locator");
        let document = state
            .rooms()
            .create_document(Some("Remote websocket owner with token".to_owned()))
            .expect("document should be created");

        let mut headers = HeaderMap::new();
        headers.insert(ORIGIN, HeaderValue::from_static(DEFAULT_FRONTEND_ORIGIN));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", document.access_token()))
                .expect("document authorization header should be valid"),
        );
        let request_uri: Uri = format!(
            "/ws/{}?access_token={}&source=edge",
            document.id,
            document.access_token()
        )
        .parse()
        .expect("websocket request URI should parse");
        let error = match resolve_websocket_room(
            &state,
            &headers,
            &request_uri,
            &document.id.to_string(),
        ) {
            Ok(_) => panic!("non-local owner should reject websocket room resolution"),
            Err(error) => error,
        };

        match error {
            AppError::RemoteOwner { redirect_url, .. } => {
                assert_eq!(
                    redirect_url,
                    Some(format!("http://node-b.internal:4000/ws/{}", document.id))
                );
            }
            other => panic!("expected conflict, received {other:?}"),
        }
    }
}
