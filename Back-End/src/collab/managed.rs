use std::time::Duration;

use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    collab::coordinator::PersistedRoomCoordinatorState,
    config::normalize_http_base_url,
    http_client::{BlockingHttpClient, RequestBuilder, RequestError, Response},
};

#[derive(Debug, Error)]
pub(crate) enum ManagedCoordinationClientError {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Request(String),
    #[error("managed coordination service reported an active lease conflict")]
    Conflict(Box<Option<PersistedRoomCoordinatorState>>),
}

#[derive(Debug, Clone)]
pub(crate) struct ManagedCoordinationClient {
    base_url: String,
    auth_token: Option<String>,
    client: BlockingHttpClient,
}

#[derive(Debug, Serialize)]
struct AcquireLeaseRequest<'a> {
    node_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<&'a str>,
    lease_ttl_secs: u64,
}

#[derive(Debug, Serialize)]
struct RenewLeaseRequest<'a> {
    node_id: &'a str,
    lease_id: Uuid,
    epoch: u64,
    lease_ttl_secs: u64,
}

#[derive(Debug, Serialize)]
struct ReleaseLeaseRequest<'a> {
    node_id: &'a str,
    lease_id: Uuid,
    epoch: u64,
}

impl ManagedCoordinationClient {
    pub(crate) fn new(
        base_url: impl Into<String>,
        auth_token: Option<String>,
        timeout: Duration,
    ) -> Result<Self, ManagedCoordinationClientError> {
        if timeout.is_zero() {
            return Err(ManagedCoordinationClientError::Config(
                "ROOM_COORDINATION_MANAGED_TIMEOUT_SECS must be greater than zero when ROOM_LOCATOR=managed or ROOM_COORDINATOR=managed".to_owned(),
            ));
        }

        let base_url =
            normalize_http_base_url(base_url.into().trim(), "ROOM_COORDINATION_MANAGED_BASE_URL")
                .map_err(ManagedCoordinationClientError::Config)?;
        let client = BlockingHttpClient::new(timeout);

        Ok(Self {
            base_url,
            auth_token,
            client,
        })
    }

    pub(crate) fn lookup_lease(
        &self,
        doc_id: &Uuid,
    ) -> Result<Option<PersistedRoomCoordinatorState>, ManagedCoordinationClientError> {
        let request = self.authorized_request("GET", &self.lease_url(doc_id));
        match request.call() {
            Ok(response) => self
                .parse_state_response(
                    response,
                    format!(
                        "failed to decode managed coordination lease lookup response for document `{doc_id}`"
                    ),
                )
                .map(Some),
            Err(RequestError::Status(404, _)) => Ok(None),
            Err(RequestError::Status(status, response)) => Err(self.unexpected_status(
                status,
                response,
                format!("managed coordination lease lookup failed for document `{doc_id}`"),
            )),
            Err(RequestError::Transport(error)) => Err(ManagedCoordinationClientError::Request(
                format!("failed to query managed coordination lease for document `{doc_id}`: {error}"),
            )),
        }
    }

    pub(crate) fn acquire_lease(
        &self,
        doc_id: &Uuid,
        node_id: &str,
        base_url: Option<&str>,
        lease_ttl: Duration,
    ) -> Result<PersistedRoomCoordinatorState, ManagedCoordinationClientError> {
        let request = self.authorized_request("POST", &self.action_url(doc_id, "acquire"));
        match request.send_json(serde_json::json!(AcquireLeaseRequest {
            node_id,
            base_url,
            lease_ttl_secs: lease_ttl.as_secs(),
        })) {
            Ok(response) => self.parse_state_response(
                response,
                format!(
                    "failed to decode managed coordination acquire response for document `{doc_id}`"
                ),
            ),
            Err(RequestError::Status(409, response)) => Err(
                ManagedCoordinationClientError::Conflict(Box::new(self.try_parse_state(response))),
            ),
            Err(RequestError::Status(status, response)) => Err(self.unexpected_status(
                status,
                response,
                format!("managed coordination acquire failed for document `{doc_id}`"),
            )),
            Err(RequestError::Transport(error)) => {
                Err(ManagedCoordinationClientError::Request(format!(
                    "failed to acquire managed coordination lease for document `{doc_id}`: {error}"
                )))
            }
        }
    }

    pub(crate) fn renew_lease(
        &self,
        doc_id: &Uuid,
        node_id: &str,
        lease_id: Uuid,
        epoch: u64,
        lease_ttl: Duration,
    ) -> Result<Option<PersistedRoomCoordinatorState>, ManagedCoordinationClientError> {
        let request = self.authorized_request("POST", &self.action_url(doc_id, "renew"));
        match request.send_json(serde_json::json!(RenewLeaseRequest {
            node_id,
            lease_id,
            epoch,
            lease_ttl_secs: lease_ttl.as_secs(),
        })) {
            Ok(response) => self
                .parse_state_response(
                    response,
                    format!(
                        "failed to decode managed coordination renew response for document `{doc_id}`"
                    ),
                )
                .map(Some),
            Err(RequestError::Status(404 | 409, _)) => Ok(None),
            Err(RequestError::Status(status, response)) => Err(self.unexpected_status(
                status,
                response,
                format!("managed coordination renew failed for document `{doc_id}`"),
            )),
            Err(RequestError::Transport(error)) => Err(ManagedCoordinationClientError::Request(
                format!(
                    "failed to renew managed coordination lease for document `{doc_id}`: {error}"
                ),
            )),
        }
    }

    pub(crate) fn release_lease(
        &self,
        doc_id: &Uuid,
        node_id: &str,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<bool, ManagedCoordinationClientError> {
        let request = self.authorized_request("POST", &self.action_url(doc_id, "release"));
        match request.send_json(serde_json::json!(ReleaseLeaseRequest {
            node_id,
            lease_id,
            epoch,
        })) {
            Ok(_) => Ok(true),
            Err(RequestError::Status(404 | 409, _)) => Ok(false),
            Err(RequestError::Status(status, response)) => Err(self.unexpected_status(
                status,
                response,
                format!("managed coordination release failed for document `{doc_id}`"),
            )),
            Err(RequestError::Transport(error)) => {
                Err(ManagedCoordinationClientError::Request(format!(
                    "failed to release managed coordination lease for document `{doc_id}`: {error}"
                )))
            }
        }
    }

    fn authorized_request(&self, method: &str, url: &str) -> RequestBuilder {
        let request = self
            .client
            .request(method, url)
            .expect("managed coordination URLs should be valid");
        match self.auth_token.as_deref() {
            Some(token) => request.set("Authorization", &format!("Bearer {token}")),
            None => request,
        }
    }

    fn lease_url(&self, doc_id: &Uuid) -> String {
        format!("{}/v1/leases/{doc_id}", self.base_url.trim_end_matches('/'))
    }

    fn action_url(&self, doc_id: &Uuid, action: &str) -> String {
        format!(
            "{}/v1/leases/{doc_id}/{action}",
            self.base_url.trim_end_matches('/')
        )
    }

    fn parse_state_response(
        &self,
        response: Response,
        context: String,
    ) -> Result<PersistedRoomCoordinatorState, ManagedCoordinationClientError> {
        response
            .into_json()
            .map_err(|error| ManagedCoordinationClientError::Request(format!("{context}: {error}")))
    }

    fn try_parse_state(&self, response: Response) -> Option<PersistedRoomCoordinatorState> {
        response.into_json().ok()
    }

    fn unexpected_status(
        &self,
        status: u16,
        response: Response,
        context: String,
    ) -> ManagedCoordinationClientError {
        let detail = response
            .into_sanitized_error_body()
            .map(|body| format!("unexpected HTTP {status}: {body}"))
            .unwrap_or_else(|| format!("unexpected HTTP {status}"));
        ManagedCoordinationClientError::Request(format!("{context}: {detail}"))
    }
}
