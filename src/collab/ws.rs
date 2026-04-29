use std::sync::Arc;

use axum::{
    extract::{
        OriginalUri, Path, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, Uri, header::ORIGIN},
    response::IntoResponse,
};
use futures_util::StreamExt;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;
use yrs_axum::ws::{AxumSink, AxumStream};

use crate::{
    collab::coordinator::RoomCoordinator,
    collab::protocol::ValidatingProtocol,
    collab::rooms::{Room, RoomRegistry},
    errors::{AppError, AppResult},
    state::AppState,
};

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

    if active_sessions == 1 {
        if let Err(error) = coordinator.room_activated(&doc_id) {
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
    }

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
    use axum::http::HeaderValue;

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
}
