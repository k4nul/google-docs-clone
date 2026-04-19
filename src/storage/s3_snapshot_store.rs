use std::time::Duration;

use axum::http::StatusCode;
use s3::{AddressingStyle, Auth, BlockingClient, Credentials, Error as S3Error};
use tracing::warn;
use uuid::Uuid;

use crate::{
    config::normalize_http_base_url,
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct S3SnapshotStore {
    endpoint: String,
    bucket: String,
    prefix: String,
    client: BlockingClient,
}

impl S3SnapshotStore {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        endpoint: impl Into<String>,
        region: impl Into<String>,
        bucket: impl Into<String>,
        prefix: impl Into<String>,
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        session_token: Option<String>,
        timeout: Duration,
        path_style: bool,
    ) -> Result<Self, StorageError> {
        if timeout.is_zero() {
            return Err(StorageError::Config(
                "SNAPSHOT_S3_TIMEOUT_SECS must be greater than zero when SNAPSHOT_STORE=s3"
                    .to_owned(),
            ));
        }

        let endpoint = normalize_http_base_url(endpoint.into().trim(), "SNAPSHOT_S3_ENDPOINT")
            .map_err(StorageError::Config)?;
        let region = normalize_required_string(region, "SNAPSHOT_S3_REGION")?;
        let bucket = normalize_required_string(bucket, "SNAPSHOT_S3_BUCKET")?;
        let prefix = normalize_prefix(prefix.into());
        let access_key_id = normalize_required_string(access_key_id, "SNAPSHOT_S3_ACCESS_KEY_ID")?;
        let secret_access_key =
            normalize_required_string(secret_access_key, "SNAPSHOT_S3_SECRET_ACCESS_KEY")?;
        let session_token = normalize_optional_string(session_token);

        let mut credentials =
            Credentials::new(access_key_id, secret_access_key).map_err(map_s3_config_error)?;
        if let Some(session_token) = session_token {
            credentials = credentials
                .with_session_token(session_token)
                .map_err(map_s3_config_error)?;
        }

        let client = BlockingClient::builder(&endpoint)
            .map_err(map_s3_config_error)?
            .region(region)
            .auth(Auth::Static(credentials))
            .addressing_style(if path_style {
                AddressingStyle::Path
            } else {
                AddressingStyle::Auto
            })
            .timeout(timeout)
            .build()
            .map_err(map_s3_config_error)?;

        Ok(Self {
            endpoint,
            bucket,
            prefix,
            client,
        })
    }

    fn object_key(&self, doc_id: &Uuid) -> String {
        format!("{}{doc_id}.json", self.prefix)
    }

    fn doc_id_from_object_key(&self, key: &str) -> Option<Uuid> {
        key.strip_prefix(&self.prefix)
            .and_then(|suffix| suffix.strip_suffix(".json"))
            .and_then(|value| Uuid::parse_str(value).ok())
    }

    fn snapshot_from_bytes(
        &self,
        expected_doc_id: Uuid,
        bytes: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn map_operation_error(&self, context: &str, error: S3Error) -> StorageError {
        match error {
            S3Error::InvalidConfig { message } | S3Error::Signing { message } => {
                StorageError::Config(format!("{context}: {message}"))
            }
            other => StorageError::Io(format!(
                "{context} (endpoint `{}`, bucket `{}`): {other}",
                self.endpoint, self.bucket
            )),
        }
    }
}

impl SnapshotStore for S3SnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = self.object_key(doc_id);
        match self.client.objects().get(&self.bucket, &key).send() {
            Ok(output) => {
                let bytes = output.bytes().map_err(|error| {
                    self.map_operation_error("failed to read s3 snapshot body", error)
                })?;
                self.snapshot_from_bytes(*doc_id, &bytes).map(Some)
            }
            Err(error) if error.status() == Some(StatusCode::NOT_FOUND) => Ok(None),
            Err(error) => Err(self
                .map_operation_error(&format!("failed to load s3 snapshot object `{key}`"), error)),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = self.object_key(&doc_id);
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize s3 snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.client
            .objects()
            .put(&self.bucket, &key)
            .content_type("application/json")
            .body_bytes(bytes)
            .send()
            .map(|_| ())
            .map_err(|error| {
                self.map_operation_error(
                    &format!("failed to save s3 snapshot object `{key}`"),
                    error,
                )
            })
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = self.object_key(doc_id);
        match self.client.objects().delete(&self.bucket, &key).send() {
            Ok(_) => Ok(()),
            Err(error) if error.status() == Some(StatusCode::NOT_FOUND) => Ok(()),
            Err(error) => Err(self.map_operation_error(
                &format!("failed to delete s3 snapshot object `{key}`"),
                error,
            )),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut request = self.client.objects().list_v2(&self.bucket).max_keys(1000);
        if !self.prefix.is_empty() {
            request = request.prefix(self.prefix.clone());
        }

        let mut documents = Vec::new();
        for page in request.pager() {
            let page = page.map_err(|error| {
                self.map_operation_error("failed to list s3 snapshot objects", error)
            })?;
            for object in page.contents {
                let Some(doc_id) = self.doc_id_from_object_key(&object.key) else {
                    continue;
                };

                match self.load_snapshot(&doc_id) {
                    Ok(Some(snapshot)) => documents.push(snapshot.document),
                    Ok(None) => continue,
                    Err(StorageError::CorruptSnapshot(doc_id)) => warn!(
                        doc_id = %doc_id,
                        key = object.key,
                        endpoint = %self.endpoint,
                        bucket = %self.bucket,
                        "skipping corrupt s3 snapshot object while building document catalog"
                    ),
                    Err(error) => return Err(error),
                }
            }
        }

        Ok(documents)
    }
}

fn normalize_required_string(
    value: impl Into<String>,
    field_name: &str,
) -> Result<String, StorageError> {
    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(StorageError::Config(format!(
            "{field_name} cannot be empty when SNAPSHOT_STORE=s3"
        )))
    } else {
        Ok(trimmed.to_owned())
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

fn normalize_prefix(value: String) -> String {
    let trimmed = value.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}/")
    }
}

fn map_s3_config_error(error: S3Error) -> StorageError {
    match error {
        S3Error::InvalidConfig { message } | S3Error::Signing { message } => {
            StorageError::Config(message)
        }
        other => StorageError::Io(other.to_string()),
    }
}
