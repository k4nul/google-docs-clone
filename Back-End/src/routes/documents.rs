use axum::{
    Json,
    extract::{OriginalUri, Path, State},
    http::{HeaderMap, StatusCode, Uri},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yrs::{Doc, GetString, ReadTxn, Transact, Update, updates::decoder::Decode};

use crate::{
    auth::{require_admin_token, require_document_access},
    collab::rooms::Room,
    errors::{AppError, AppResult},
    http_params::parse_uuid_param,
    models::access::DocumentCredentials,
    models::document::Document,
    state::AppState,
    storage::{DocumentSnapshot, StorageError},
};

#[derive(Debug, Serialize)]
pub struct DocumentsResponse {
    pub documents: Vec<DocumentListItem>,
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
    pub title: Option<String>,
    pub hide_preview: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DocumentListItem {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub hide_preview: bool,
    pub preview_hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

pub async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<DocumentsResponse>> {
    require_admin_token(&headers, state.api_token())?;

    let documents = state
        .rooms()
        .list_document_snapshots()
        .map(|snapshots| {
            snapshots
                .into_iter()
                .map(document_list_item_from_snapshot)
                .collect()
        })
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
    let title = match payload.title {
        Some(title) => Some(normalized_title(title)?),
        None => None,
    };

    if title.is_none() && payload.hide_preview.is_none() {
        return Err(AppError::BadRequest(
            "title or hide_preview must be provided".to_owned(),
        ));
    }

    let document = state
        .rooms()
        .update_document(&id, title, payload.hide_preview)
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

fn document_list_item_from_snapshot(snapshot: DocumentSnapshot) -> DocumentListItem {
    let DocumentSnapshot { document, update } = snapshot;
    let preview_hidden = document.hide_preview;
    let preview = if preview_hidden {
        None
    } else {
        preview_from_update(update.as_slice())
    };

    DocumentListItem {
        id: document.id,
        title: document.title,
        created_at: document.created_at,
        updated_at: document.updated_at,
        hide_preview: document.hide_preview,
        preview_hidden,
        preview,
    }
}

fn preview_from_update(update: &[u8]) -> Option<String> {
    let doc = Doc::new();
    let mut txn = doc.transact_mut();
    let update = Update::decode_v1(update).ok()?;
    txn.apply_update(update);
    drop(txn);

    let txn = doc.transact();
    if let Some(preview) = txn.get_xml_fragment("content").and_then(|content| {
        normalize_preview_text(&plain_text_from_markup(&content.get_string(&txn)))
    }) {
        return Some(preview);
    }

    txn.get_text("content").and_then(|content| {
        normalize_preview_text(&plain_text_from_markup(&content.get_string(&txn)))
    })
}

fn plain_text_from_markup(value: &str) -> String {
    let mut text = String::with_capacity(value.len());
    let mut in_tag = false;
    let mut previous_was_space = false;

    for character in value.chars() {
        match character {
            '<' => {
                in_tag = true;
                if !previous_was_space {
                    text.push(' ');
                    previous_was_space = true;
                }
            }
            '>' => {
                in_tag = false;
            }
            _ if in_tag => {}
            _ if character.is_whitespace() => {
                if !previous_was_space {
                    text.push(' ');
                    previous_was_space = true;
                }
            }
            _ => {
                text.push(character);
                previous_was_space = false;
            }
        }
    }

    decode_basic_entities(text.trim())
}

fn decode_basic_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn normalize_preview_text(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");

    if normalized.is_empty() {
        return None;
    }

    Some(truncate_preview(&normalized, 180))
}

fn truncate_preview(value: &str, max_chars: usize) -> String {
    let mut characters = value.chars();
    let preview = characters.by_ref().take(max_chars).collect::<String>();

    if characters.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use yrs::{StateVector, Text, XmlFragment, XmlTextPrelim};

    fn xml_snapshot(hide_preview: bool, body: &str) -> DocumentSnapshot {
        let mut document = Document::new(Uuid::new_v4(), Some("Preview test".to_owned()));
        document.set_hide_preview(hide_preview);

        let doc = Doc::new();
        let content = doc.get_or_insert_xml_fragment("content");
        let mut txn = doc.transact_mut();
        content.push_back(&mut txn, XmlTextPrelim::new(body));
        let update = txn.encode_state_as_update_v1(&StateVector::default());

        DocumentSnapshot::new(document, update)
    }

    fn text_update(body: &str) -> Vec<u8> {
        let doc = Doc::new();
        let content = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        content.insert(&mut txn, 0, body);
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    #[test]
    fn document_list_item_includes_plain_text_preview_from_xml_content() {
        let item = document_list_item_from_snapshot(xml_snapshot(
            false,
            "Launch notes & private follow-up",
        ));

        assert_eq!(
            item.preview.as_deref(),
            Some("Launch notes & private follow-up")
        );
        assert!(!item.preview_hidden);
    }

    #[test]
    fn document_list_item_redacts_hidden_preview_content() {
        let item = document_list_item_from_snapshot(xml_snapshot(true, "Sensitive launch plan"));

        assert!(item.preview.is_none());
        assert!(item.preview_hidden);
        assert!(item.hide_preview);
    }

    #[test]
    fn preview_extraction_falls_back_to_text_content_root() {
        assert_eq!(
            preview_from_update(text_update("Plain text root content").as_slice()).as_deref(),
            Some("Plain text root content")
        );
    }

    #[test]
    fn preview_extraction_strips_xml_markup_and_decodes_basic_entities() {
        assert_eq!(
            normalize_preview_text(&plain_text_from_markup(
                "<h2>Title</h2><p>Body &amp; notes</p>",
            ))
            .as_deref(),
            Some("Title Body & notes")
        );
    }
}
