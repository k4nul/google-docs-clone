use axum::{
    Json,
    extract::{OriginalUri, Path, State},
    http::{HeaderMap, StatusCode, Uri},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{require_admin_token, require_document_access},
    collab::rooms::Room,
    errors::{AppError, AppResult},
    http_params::parse_uuid_param,
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

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: String,
}

pub async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<DocumentsResponse>> {
    require_admin_token(&headers, state.api_token())?;

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
    require_admin_token(&headers, state.api_token())?;

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
    let room = accessible_room(&state, &original_uri, id)?;
    require_document_access(&headers, &room, id)?;

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
    let room = accessible_room(&state, &original_uri, id)?;
    require_document_access(&headers, &room, id)?;
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

pub async fn update_document(
    Path(raw_id): Path<String>,
    OriginalUri(original_uri): OriginalUri,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateDocumentRequest>,
) -> AppResult<Json<DocumentResponse>> {
    let id = parse_uuid_param("id", &raw_id)?;
    let room = accessible_room(&state, &original_uri, id)?;
    require_document_access(&headers, &room, id)?;
    let title = normalized_title(payload.title)?;
    let document = state
        .rooms()
        .rename_document(&id, title)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("document `{id}` was not found")))?;

    Ok(Json(DocumentResponse { document }))
}

fn normalized_title(title: String) -> AppResult<String> {
    let title = title.trim().to_owned();

    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".to_owned()));
    }

    Ok(title)
}

fn accessible_room(
    state: &AppState,
    request_uri: &Uri,
    id: Uuid,
) -> AppResult<std::sync::Arc<Room>> {
    state.ensure_local_room_owner_for_request(&id, request_uri)?;
    let room = state
        .rooms()
        .get_or_restore(&id)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("document `{id}` was not found")))?;

    Ok(room)
}
