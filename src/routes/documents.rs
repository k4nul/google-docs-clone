use axum::{
    Json,
    extract::{OriginalUri, Path, State},
    http::{HeaderMap, StatusCode, Uri},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::require_bearer_token,
    collab::rooms::Room,
    errors::{AppError, AppResult},
    models::access::DocumentCredentials,
    models::document::Document,
    state::AppState,
    storage::StorageError,
};

#[derive(Debug, Serialize)]
pub struct DocumentsResponse {
    pub documents: Vec<Document>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub document: Document,
}

#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub document: Document,
    pub credentials: DocumentCredentials,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: Option<String>,
}

pub async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<DocumentsResponse>> {
    require_admin_token(&state, &headers)?;

    let documents = state
        .rooms()
        .list_documents()
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?;

    Ok(Json(DocumentsResponse { documents }))
}

pub async fn create_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDocumentRequest>,
) -> AppResult<(StatusCode, Json<CreateDocumentResponse>)> {
    require_admin_token(&state, &headers)?;

    let document = state
        .rooms()
        .create_document(payload.title)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?;
    let credentials = DocumentCredentials {
        access_token: document.access_token().to_owned(),
    };

    Ok((
        StatusCode::CREATED,
        Json(CreateDocumentResponse {
            document,
            credentials,
        }),
    ))
}

pub async fn get_document(
    Path(raw_id): Path<String>,
    OriginalUri(original_uri): OriginalUri,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<DocumentResponse>> {
    let id = parse_uuid_param("id", &raw_id)?;
    let room = authorized_room(&state, &headers, &original_uri, id)?;

    Ok(Json(DocumentResponse {
        document: room.document(),
    }))
}

pub async fn delete_document(
    Path(raw_id): Path<String>,
    OriginalUri(original_uri): OriginalUri,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let id = parse_uuid_param("id", &raw_id)?;
    authorized_room(&state, &headers, &original_uri, id)?;
    state
        .rooms()
        .delete_document(&id)
        .map_err(|error| match error {
            StorageError::DocumentBusy(doc_id) => AppError::Conflict(format!(
                "document `{doc_id}` cannot be deleted while collaboration sessions are active"
            )),
            other => AppError::from(anyhow::Error::from(other)),
        })?
        .ok_or_else(|| AppError::NotFound(format!("document `{id}` was not found")))?;

    Ok(StatusCode::NO_CONTENT)
}

fn require_admin_token(state: &AppState, headers: &HeaderMap) -> AppResult<()> {
    let token = require_bearer_token(headers)?;

    if token != state.api_token() {
        return Err(AppError::Forbidden(
            "provided API token does not grant this operation".to_owned(),
        ));
    }

    Ok(())
}

fn authorized_room(
    state: &AppState,
    headers: &HeaderMap,
    request_uri: &Uri,
    id: Uuid,
) -> AppResult<std::sync::Arc<Room>> {
    let token = require_bearer_token(headers)?;
    state.ensure_local_room_owner_for_request(&id, request_uri)?;
    let room = state
        .rooms()
        .get_or_restore(&id)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("document `{id}` was not found")))?;

    if !room.authorizes(token) {
        return Err(AppError::Forbidden(format!(
            "provided token does not grant access to document `{id}`"
        )));
    }

    Ok(room)
}

fn parse_uuid_param(parameter: &str, raw_value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(raw_value).map_err(|_| {
        AppError::BadRequest(format!(
            "{parameter} must be a valid UUID, received `{raw_value}`"
        ))
    })
}
