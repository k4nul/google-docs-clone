use std::{io, time::Duration};

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tracing::warn;
use url::Url;
use uuid::Uuid;

use crate::{
    config::normalize_http_base_url,
    http_client::{BlockingHttpClient, RequestError, Response},
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

type HmacSha256 = Hmac<Sha256>;

pub struct S3SnapshotStore {
    endpoint: Url,
    endpoint_label: String,
    region: String,
    bucket: String,
    prefix: String,
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
    path_style: bool,
    client: BlockingHttpClient,
}

struct RequestTarget {
    url: Url,
    canonical_uri: String,
    canonical_query: String,
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

        let endpoint_label =
            normalize_http_base_url(endpoint.into().trim(), "SNAPSHOT_S3_ENDPOINT")
                .map_err(StorageError::Config)?;
        let endpoint = Url::parse(&endpoint_label)
            .map_err(|error| StorageError::Config(format!("SNAPSHOT_S3_ENDPOINT: {error}")))?;
        let region = normalize_required_string(region, "SNAPSHOT_S3_REGION")?;
        let bucket = normalize_required_string(bucket, "SNAPSHOT_S3_BUCKET")?;
        let prefix = normalize_prefix(prefix.into());
        let access_key_id = normalize_required_string(access_key_id, "SNAPSHOT_S3_ACCESS_KEY_ID")?;
        let secret_access_key =
            normalize_required_string(secret_access_key, "SNAPSHOT_S3_SECRET_ACCESS_KEY")?;
        let session_token = normalize_optional_string(session_token);
        let client = BlockingHttpClient::new(timeout);

        Ok(Self {
            endpoint,
            endpoint_label,
            region,
            bucket,
            prefix,
            access_key_id,
            secret_access_key,
            session_token,
            path_style,
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

    fn object_target(&self, key: &str) -> Result<RequestTarget, StorageError> {
        self.request_target(
            if self.path_style {
                vec![self.bucket.clone()]
            } else {
                Vec::new()
            },
            key.split('/').map(str::to_owned).collect(),
            Vec::new(),
        )
    }

    fn list_target(&self) -> Result<RequestTarget, StorageError> {
        let mut query = vec![
            ("list-type".to_owned(), "2".to_owned()),
            ("max-keys".to_owned(), "1000".to_owned()),
        ];
        if !self.prefix.is_empty() {
            query.push(("prefix".to_owned(), self.prefix.clone()));
        }

        self.request_target(
            if self.path_style {
                vec![self.bucket.clone()]
            } else {
                Vec::new()
            },
            Vec::new(),
            query,
        )
    }

    fn request_target(
        &self,
        mut path_segments: Vec<String>,
        extra_segments: Vec<String>,
        mut query: Vec<(String, String)>,
    ) -> Result<RequestTarget, StorageError> {
        let mut url = self.endpoint.clone();
        if !self.path_style {
            let host = url.host_str().ok_or_else(|| {
                StorageError::Config("SNAPSHOT_S3_ENDPOINT must include a host".to_owned())
            })?;
            let bucket_host = format!("{}.{}", self.bucket, host);
            url.set_host(Some(&bucket_host)).map_err(|_| {
                StorageError::Config("invalid SNAPSHOT_S3_BUCKET host label".to_owned())
            })?;
        }

        let base_segments = url
            .path_segments()
            .map(|segments| {
                segments
                    .filter(|segment| !segment.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut canonical_segments = base_segments;
        canonical_segments.append(&mut path_segments);
        canonical_segments.extend(extra_segments);

        let canonical_uri = canonical_path(&canonical_segments);
        url.set_path(&canonical_uri);

        query.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
        let canonical_query = query
            .iter()
            .map(|(key, value)| {
                format!(
                    "{}={}",
                    encode_query_component(key),
                    encode_query_component(value)
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        if canonical_query.is_empty() {
            url.set_query(None);
        } else {
            url.set_query(Some(&canonical_query));
        }

        Ok(RequestTarget {
            url,
            canonical_uri,
            canonical_query,
        })
    }

    fn send(
        &self,
        method: &str,
        target: &RequestTarget,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<Response, StorageError> {
        let payload_hash = hex::encode(Sha256::digest(body));
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        let host_header = host_header_value(&target.url)?;

        let mut canonical_headers = vec![
            format!("host:{host_header}"),
            format!("x-amz-content-sha256:{payload_hash}"),
            format!("x-amz-date:{amz_date}"),
        ];
        let mut signed_headers = vec![
            "host".to_owned(),
            "x-amz-content-sha256".to_owned(),
            "x-amz-date".to_owned(),
        ];
        if let Some(session_token) = self.session_token.as_deref() {
            canonical_headers.push(format!("x-amz-security-token:{session_token}"));
            signed_headers.push("x-amz-security-token".to_owned());
        }

        let canonical_request = format!(
            "{method}\n{}\n{}\n{}\n{}\n{}",
            target.canonical_uri,
            target.canonical_query,
            canonical_headers.join("\n") + "\n",
            signed_headers.join(";"),
            payload_hash,
        );
        let credential_scope = format!("{date_stamp}/{}/s3/aws4_request", self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            hex::encode(Sha256::digest(canonical_request.as_bytes()))
        );
        let signing_key = signing_key(&self.secret_access_key, &date_stamp, &self.region)?;
        let signature = {
            let mut mac = HmacSha256::new_from_slice(&signing_key).map_err(|error| {
                StorageError::Config(format!("failed to build s3 signer: {error}"))
            })?;
            mac.update(string_to_sign.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        };
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={}, Signature={signature}",
            self.access_key_id,
            signed_headers.join(";"),
        );

        let mut request = self
            .client
            .request(method, target.url.as_str())
            .map_err(|error| {
                StorageError::Io(format!(
                    "{method} {} (endpoint `{}`, bucket `{}`): {error}",
                    target.url, self.endpoint_label, self.bucket
                ))
            })?
            .set("Authorization", &authorization)
            .set("x-amz-content-sha256", &payload_hash)
            .set("x-amz-date", &amz_date);
        if let Some(content_type) = content_type {
            request = request.set("Content-Type", content_type);
        }
        if let Some(session_token) = self.session_token.as_deref() {
            request = request.set("x-amz-security-token", session_token);
        }

        if body.is_empty() {
            request
                .call()
                .map_err(|error| self.map_request_error(method, &target.url, error))
        } else {
            request
                .send_bytes(body)
                .map_err(|error| self.map_request_error(method, &target.url, error))
        }
    }

    fn map_request_error(&self, method: &str, url: &Url, error: RequestError) -> StorageError {
        match error {
            RequestError::Status(status, response) => {
                let body = response.into_string().unwrap_or_default();
                let detail = if body.trim().is_empty() {
                    format!("unexpected HTTP {status}")
                } else {
                    format!("unexpected HTTP {status}: {}", body.trim())
                };
                StorageError::Io(format!(
                    "{method} {url} (endpoint `{}`, bucket `{}`): {detail}",
                    self.endpoint_label, self.bucket
                ))
            }
            RequestError::Transport(error) => StorageError::Io(format!(
                "{method} {url} (endpoint `{}`, bucket `{}`): {error}",
                self.endpoint_label, self.bucket
            )),
        }
    }
}

impl SnapshotStore for S3SnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = self.object_key(doc_id);
        let target = self.object_target(&key)?;
        match self.send("GET", &target, &[], None) {
            Ok(response) => {
                let bytes = read_response_bytes(response).map_err(|error| {
                    StorageError::Io(format!(
                        "failed to read s3 snapshot body for `{key}` (endpoint `{}`, bucket `{}`): {error}",
                        self.endpoint_label, self.bucket
                    ))
                })?;
                self.snapshot_from_bytes(*doc_id, &bytes).map(Some)
            }
            Err(StorageError::Io(message)) if message.contains("unexpected HTTP 404") => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = self.object_key(&doc_id);
        let target = self.object_target(&key)?;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize s3 snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.send("PUT", &target, &bytes, Some("application/json"))
            .map(|_| ())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = self.object_key(doc_id);
        let target = self.object_target(&key)?;
        match self.send("DELETE", &target, &[], None) {
            Ok(_) => Ok(()),
            Err(StorageError::Io(message)) if message.contains("unexpected HTTP 404") => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let target = self.list_target()?;
        let response = self.send("GET", &target, &[], None)?;
        let body = response.into_string().map_err(|error| {
            StorageError::Io(format!(
                "failed to read s3 snapshot catalog response (endpoint `{}`, bucket `{}`): {error}",
                self.endpoint_label, self.bucket
            ))
        })?;

        let mut documents = Vec::new();
        for key in extract_xml_values(&body, "Key") {
            let Some(doc_id) = self.doc_id_from_object_key(&key) else {
                continue;
            };

            match self.load_snapshot(&doc_id) {
                Ok(Some(snapshot)) => documents.push(snapshot.document),
                Ok(None) => continue,
                Err(StorageError::CorruptSnapshot(doc_id)) => warn!(
                    doc_id = %doc_id,
                    key,
                    endpoint = %self.endpoint_label,
                    bucket = %self.bucket,
                    "skipping corrupt s3 snapshot object while building document catalog"
                ),
                Err(error) => return Err(error),
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

fn canonical_path(segments: &[String]) -> String {
    if segments.is_empty() {
        "/".to_owned()
    } else {
        format!(
            "/{}",
            segments
                .iter()
                .map(|segment| encode_path_segment(segment))
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}

fn encode_path_segment(value: &str) -> String {
    percent_encode(value, b"-_.~")
}

fn encode_query_component(value: &str) -> String {
    percent_encode(value, b"-_.~")
}

fn percent_encode(value: &str, allowed: &[u8]) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || allowed.contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

fn host_header_value(url: &Url) -> Result<String, StorageError> {
    let host = url.host_str().ok_or_else(|| {
        StorageError::Config("SNAPSHOT_S3_ENDPOINT must include a host".to_owned())
    })?;
    Ok(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_owned(),
    })
}

fn signing_key(
    secret_access_key: &str,
    date_stamp: &str,
    region: &str,
) -> Result<Vec<u8>, StorageError> {
    let date_key = hmac_bytes(
        format!("AWS4{secret_access_key}").as_bytes(),
        date_stamp.as_bytes(),
    )?;
    let region_key = hmac_bytes(&date_key, region.as_bytes())?;
    let service_key = hmac_bytes(&region_key, b"s3")?;
    hmac_bytes(&service_key, b"aws4_request")
}

fn hmac_bytes(key: &[u8], data: &[u8]) -> Result<Vec<u8>, StorageError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|error| StorageError::Config(format!("failed to build s3 signer: {error}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn extract_xml_values(body: &str, tag: &str) -> Vec<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let mut values = Vec::new();
    let mut remaining = body;

    while let Some(start_index) = remaining.find(&start_tag) {
        let after_start = &remaining[start_index + start_tag.len()..];
        let Some(end_index) = after_start.find(&end_tag) else {
            break;
        };
        values.push(after_start[..end_index].to_owned());
        remaining = &after_start[end_index + end_tag.len()..];
    }

    values
}

fn read_response_bytes(response: Response) -> io::Result<Vec<u8>> {
    Ok(response.into_bytes())
}
