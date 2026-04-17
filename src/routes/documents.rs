use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
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

pub async fn list_documents(State(state): State<AppState>) -> Json<DocumentsResponse> {
    Json(DocumentsResponse {
        documents: state.rooms().list_documents(),
    })
}

pub async fn get_document(
    Path(raw_id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<DocumentResponse>> {
    let id = parse_uuid_param("id", &raw_id)?;
    let room = state.rooms().get_or_create(id);

    Ok(Json(DocumentResponse {
        document: room.document(),
    }))
}

fn parse_uuid_param(parameter: &str, raw_value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(raw_value).map_err(|_| {
        AppError::BadRequest(format!(
            "{parameter} must be a valid UUID, received `{raw_value}`"
        ))
    })
}
