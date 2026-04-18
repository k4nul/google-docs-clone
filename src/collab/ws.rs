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
    let doc_id = parse_uuid_param("doc_id", &raw_doc_id)?;
    validate_origin(&state, &headers, doc_id)?;
    let token = require_bearer_token(&headers)?;
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
