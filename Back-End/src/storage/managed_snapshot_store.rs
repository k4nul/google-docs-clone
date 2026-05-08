use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::{
    config::normalize_http_base_url,
    http_client::{BlockingHttpClient, RequestBuilder, RequestError, Response},
    models::document::Document,
    storage::{DocumentSnapshot, SnapshotStore, StorageError},
};

pub struct ManagedSnapshotStore {
    base_url: String,
    auth_token: Option<String>,
    client: BlockingHttpClient,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManagedSnapshotPayload {
    document: ManagedSnapshotDocument,
    update: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManagedSnapshotDocument {
    id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct ManagedSnapshotCatalogResponse {
    documents: Vec<ManagedSnapshotDocument>,
}

impl ManagedSnapshotStore {
    pub fn new(
        base_url: impl Into<String>,
        auth_token: Option<String>,
        timeout: Duration,
    ) -> Result<Self, StorageError> {
        if timeout.is_zero() {
            return Err(StorageError::Config(
                "SNAPSHOT_MANAGED_TIMEOUT_SECS must be greater than zero when SNAPSHOT_STORE=managed"
                    .to_owned(),
            ));
        }

        let base_url = normalize_http_base_url(base_url.into().trim(), "SNAPSHOT_MANAGED_BASE_URL")
            .map_err(StorageError::Config)?;
        let client = BlockingHttpClient::new(timeout);

        Ok(Self {
            base_url,
            auth_token,
            client,
        })
    }

    fn authorized_request(&self, method: &str, url: &str) -> RequestBuilder {
        let request = self
            .client
            .request(method, url)
            .expect("managed snapshot URLs should be valid");
        match self.auth_token.as_deref() {
            Some(token) => request.set("Authorization", &format!("Bearer {token}")),
            None => request,
        }
    }

    fn snapshots_url(&self) -> String {
        format!("{}/v1/snapshots", self.base_url.trim_end_matches('/'))
    }

    fn snapshot_url(&self, doc_id: &Uuid) -> String {
        format!("{}/{}", self.snapshots_url(), doc_id)
    }

    fn parse_snapshot_response(
        &self,
        response: Response,
        expected_doc_id: Uuid,
        context: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let payload: ManagedSnapshotPayload = response
            .into_json()
            .map_err(|error| StorageError::Io(format!("{context}: {error}")))?;
        payload.into_snapshot(expected_doc_id)
    }

    fn unexpected_status(&self, status: u16, response: Response, context: &str) -> StorageError {
        let body = response.into_string().unwrap_or_default();
        let detail = if body.trim().is_empty() {
            format!("unexpected HTTP {status}")
        } else {
            format!("unexpected HTTP {status}: {}", body.trim())
        };
        StorageError::Io(format!("{context}: {detail}"))
    }
}

impl ManagedSnapshotPayload {
    fn from_snapshot(snapshot: DocumentSnapshot) -> Self {
        let document = snapshot.document;
        let access_token = document.access_token().to_owned();
        Self {
            document: ManagedSnapshotDocument {
                id: document.id,
                title: document.title,
                created_at: document.created_at,
                updated_at: document.updated_at,
                access_token,
            },
            update: snapshot.update,
        }
    }

    fn into_snapshot(self, expected_doc_id: Uuid) -> Result<DocumentSnapshot, StorageError> {
        if self.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(DocumentSnapshot::new(
            self.document.into_document(expected_doc_id)?,
            self.update,
        ))
    }
}

impl ManagedSnapshotDocument {
    fn into_document(self, expected_doc_id: Uuid) -> Result<Document, StorageError> {
        if self.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(Document::from_parts(
            self.id,
            self.title,
            self.created_at,
            self.updated_at,
            self.access_token,
        ))
    }
}

impl SnapshotStore for ManagedSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let request = self.authorized_request("GET", &self.snapshot_url(doc_id));
        match request.call() {
            Ok(response) => self
                .parse_snapshot_response(
                    response,
                    *doc_id,
                    &format!("failed to decode managed snapshot response for document `{doc_id}`"),
                )
                .map(Some),
            Err(RequestError::Status(404, _)) => Ok(None),
            Err(RequestError::Status(status, response)) => Err(self.unexpected_status(
                status,
                response,
                &format!("managed snapshot lookup failed for document `{doc_id}`"),
            )),
            Err(RequestError::Transport(error)) => Err(StorageError::Io(format!(
                "failed to load managed snapshot for document `{doc_id}`: {error}"
            ))),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let request = self.authorized_request("PUT", &self.snapshot_url(&doc_id));
        match request.send_json(serde_json::json!(ManagedSnapshotPayload::from_snapshot(
            snapshot
        ))) {
            Ok(_) => Ok(()),
            Err(RequestError::Status(status, response)) => Err(self.unexpected_status(
                status,
                response,
                &format!("managed snapshot save failed for document `{doc_id}`"),
            )),
            Err(RequestError::Transport(error)) => Err(StorageError::Io(format!(
                "failed to save managed snapshot for document `{doc_id}`: {error}"
            ))),
        }
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let request = self.authorized_request("DELETE", &self.snapshot_url(doc_id));
        match request.call() {
            Ok(_) | Err(RequestError::Status(404, _)) => Ok(()),
            Err(RequestError::Status(status, response)) => Err(self.unexpected_status(
                status,
                response,
                &format!("managed snapshot delete failed for document `{doc_id}`"),
            )),
            Err(RequestError::Transport(error)) => Err(StorageError::Io(format!(
                "failed to delete managed snapshot for document `{doc_id}`: {error}"
            ))),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let request = self.authorized_request("GET", &self.snapshots_url());
        let response = match request.call() {
            Ok(response) => response,
            Err(RequestError::Status(status, response)) => {
                return Err(self.unexpected_status(
                    status,
                    response,
                    "managed snapshot catalog lookup failed",
                ));
            }
            Err(RequestError::Transport(error)) => {
                return Err(StorageError::Io(format!(
                    "failed to list managed snapshots: {error}"
                )));
            }
        };

        let catalog: ManagedSnapshotCatalogResponse = response.into_json().map_err(|error| {
            StorageError::Io(format!(
                "failed to decode managed snapshot catalog response: {error}"
            ))
        })?;

        let mut documents = Vec::new();
        for document in catalog.documents {
            let doc_id = document.id;
            match document.into_document(doc_id) {
                Ok(document) => documents.push(document),
                Err(StorageError::CorruptSnapshot(doc_id)) => warn!(
                    doc_id = %doc_id,
                    base_url = %self.base_url,
                    "skipping corrupt managed snapshot document while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
