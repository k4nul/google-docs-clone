use std::sync::Arc;

use axum::{
    extract::{
        Path, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, header::ORIGIN},
    response::IntoResponse,
};
use futures_util::StreamExt;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;
use yrs_axum::ws::{AxumSink, AxumStream};

use crate::{
    auth::require_bearer_token,
    collab::protocol::ValidatingProtocol,
    collab::rooms::{Room, RoomRegistry},
    errors::{AppError, AppResult},
    state::AppState,
};

pub async fn ws_handler(
    Path(raw_doc_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> AppResult<impl IntoResponse> {
    let (doc_id, room) = resolve_websocket_room(&state, &headers, &raw_doc_id)?;

    let registry = state.rooms_registry();
    Ok(ws.on_upgrade(move |socket| serve_room_socket(socket, registry, room, doc_id)))
}

async fn serve_room_socket(
    socket: WebSocket,
    registry: Arc<RoomRegistry>,
    room: Arc<Room>,
    doc_id: Uuid,
) {
    let active_sessions = room.start_session();
    info!(%doc_id, active_sessions, "websocket collaboration session started");

    let broadcast_group = room.broadcast_group().await;
    let (sink, stream) = socket.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);
    let subscription = broadcast_group.subscribe_with(sink, stream, ValidatingProtocol);

    match subscription.completed().await {
        Ok(()) => info!(%doc_id, "websocket collaboration session ended"),
        Err(error) => warn!(%doc_id, %error, "websocket collaboration session failed"),
    }

    match registry.persist_and_evict_if_idle(&doc_id, &room) {
        Ok(true) => info!(%doc_id, "persisted snapshot and evicted idle room"),
        Ok(false) => info!(
            %doc_id,
            active_sessions = room.active_sessions(),
            "room remains active after websocket session"
        ),
        Err(error) => warn!(%doc_id, %error, "failed to persist snapshot after websocket session"),
    }
}

fn parse_uuid_param(parameter: &str, raw_value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(raw_value).map_err(|_| {
        AppError::BadRequest(format!(
            "{parameter} must be a valid UUID, received `{raw_value}`"
        ))
    })
}

fn resolve_websocket_room(
    state: &AppState,
    headers: &HeaderMap,
    raw_doc_id: &str,
) -> AppResult<(Uuid, Arc<Room>)> {
    let doc_id = parse_uuid_param("doc_id", raw_doc_id)?;
    validate_origin(state, headers, doc_id)?;
    let token = require_bearer_token(headers)?;
    state.ensure_local_room_owner(&doc_id)?;
    let room = state
        .rooms()
        .get_or_restore(&doc_id)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("document `{doc_id}` was not found")))?;

    if !room.authorizes(token) {
        warn!(%doc_id, "rejected websocket connection with invalid document token");
        return Err(AppError::Forbidden(format!(
            "provided token does not grant access to document `{doc_id}`"
        )));
    }

    Ok((doc_id, room))
}

fn validate_origin(state: &AppState, headers: &HeaderMap, doc_id: Uuid) -> AppResult<()> {
    let Some(origin) = headers.get(ORIGIN) else {
        warn!(%doc_id, "rejected websocket connection without origin header");
        return Err(AppError::Forbidden(
            "Origin header is required for websocket connections".to_owned(),
        ));
    };

    if origin.as_bytes() != state.frontend_origin().as_bytes() {
        let received_origin = origin.to_str().unwrap_or("<invalid origin header>");
        warn!(
            %doc_id,
            received_origin,
            allowed_origin = state.frontend_origin(),
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
    use axum::http::{HeaderValue, header::AUTHORIZATION};

    use crate::{
        collab::locator::{ResolvedRoom, RoomLocator, RoomLocatorError, RoomOwnerHint},
        config::DEFAULT_FRONTEND_ORIGIN,
        storage::InMemorySnapshotStore,
    };

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
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", document.access_token()))
                .expect("authorization header should be valid"),
        );

        let error = match resolve_websocket_room(&state, &headers, &document.id.to_string()) {
            Ok(_) => panic!("non-local owner should reject websocket room resolution"),
            Err(error) => error,
        };

        match error {
            AppError::RemoteOwner {
                message,
                owner_node_id,
                owner_base_url,
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
            }
            other => panic!("expected conflict, received {other:?}"),
        }
    }
}
