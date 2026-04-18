use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::document::Document,
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct DocumentsResponse {
    pub documents: Vec<Document>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub document: Document,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: Option<String>,
}

pub async fn list_documents(State(state): State<AppState>) -> Json<DocumentsResponse> {
    Json(DocumentsResponse {
        documents: state.rooms().list_documents(),
    })
}

pub async fn create_document(
    State(state): State<AppState>,
    Json(payload): Json<CreateDocumentRequest>,
) -> AppResult<(StatusCode, Json<DocumentResponse>)> {
    let document = state
        .rooms()
        .create_document(payload.title)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(DocumentResponse { document })))
}

pub async fn get_document(
    Path(raw_id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<DocumentResponse>> {
    let id = parse_uuid_param("id", &raw_id)?;
    let room = state
        .rooms()
        .get_or_restore(&id)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("document `{id}` was not found")))?;

    Ok(Json(DocumentResponse {
        document: room.document(),
    }))
}

pub async fn delete_document(
    Path(raw_id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    let id = parse_uuid_param("id", &raw_id)?;
    state
        .rooms()
        .delete_document(&id)
        .map_err(anyhow::Error::from)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("document `{id}` was not found")))?;

    Ok(StatusCode::NO_CONTENT)
}

fn parse_uuid_param(parameter: &str, raw_value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(raw_value).map_err(|_| {
        AppError::BadRequest(format!(
            "{parameter} must be a valid UUID, received `{raw_value}`"
        ))
    })
}
