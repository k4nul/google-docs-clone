use axum::{
    Json,
    http::{HeaderValue, StatusCode, Uri, header::LOCATION},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("conflict: {message}")]
    RemoteOwner {
        message: String,
        owner_node_id: String,
        owner_base_url: Option<String>,
        redirect_url: Option<String>,
    },
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<OwnerBody>,
}

#[derive(Debug, Serialize)]
struct OwnerBody {
    node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
}

impl AppError {
    pub fn with_redirect_from_request(self, request_uri: &Uri) -> Self {
        match self {
            AppError::RemoteOwner {
                message,
                owner_node_id,
                owner_base_url,
                redirect_url,
            } => {
                let redirect_url = redirect_url.or_else(|| {
                    owner_base_url
                        .as_deref()
                        .map(|base_url| build_redirect_url(base_url, request_uri))
                });

                AppError::RemoteOwner {
                    message,
                    owner_node_id,
                    owner_base_url,
                    redirect_url,
                }
            }
            other => other,
        }
    }
}

fn build_redirect_url(base_url: &str, request_uri: &Uri) -> String {
    let path_and_query = request_uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");
    format!("{base_url}{path_and_query}")
}

fn header_value_or_log(header_name: &str, value: &str) -> Option<HeaderValue> {
    match HeaderValue::from_str(value) {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::warn!(
                header_name,
                value = %value,
                %error,
                "skipped invalid response header value"
            );
            None
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    error: "bad_request",
                    message,
                    owner: None,
                }),
            )
                .into_response(),
            AppError::Config(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: "config_error",
                    message,
                    owner: None,
                }),
            )
                .into_response(),
            AppError::Conflict(message) => (
                StatusCode::CONFLICT,
                Json(ErrorBody {
                    error: "conflict",
                    message,
                    owner: None,
                }),
            )
                .into_response(),
            AppError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                Json(ErrorBody {
                    error: "forbidden",
                    message,
                    owner: None,
                }),
            )
                .into_response(),
            AppError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "unauthorized",
                    message,
                    owner: None,
                }),
            )
                .into_response(),
            AppError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "not_found",
                    message,
                    owner: None,
                }),
            )
                .into_response(),
            AppError::RemoteOwner {
                message,
                owner_node_id,
                owner_base_url,
                redirect_url,
            } => {
                let mut response = (
                    StatusCode::CONFLICT,
                    Json(ErrorBody {
                        error: "conflict",
                        message,
                        owner: Some(OwnerBody {
                            node_id: owner_node_id.clone(),
                            base_url: owner_base_url.clone(),
                        }),
                    }),
                )
                    .into_response();

                if let Some(value) =
                    header_value_or_log("x-collab-owner-node-id", owner_node_id.as_str())
                {
                    response
                        .headers_mut()
                        .insert("x-collab-owner-node-id", value);
                }

                if let Some(owner_base_url) = owner_base_url.as_deref() {
                    if let Some(value) =
                        header_value_or_log("x-collab-owner-base-url", owner_base_url)
                    {
                        response
                            .headers_mut()
                            .insert("x-collab-owner-base-url", value);
                    }
                }

                if let Some(redirect_url) = redirect_url.as_deref() {
                    if let Some(value) =
                        header_value_or_log("x-collab-redirect-location", redirect_url)
                    {
                        response
                            .headers_mut()
                            .insert("x-collab-redirect-location", value);
                    }

                    if let Some(value) = header_value_or_log("location", redirect_url) {
                        response.headers_mut().insert(LOCATION, value);
                    }
                }

                response
            }
            AppError::Internal(error) => {
                tracing::error!(%error, "unexpected internal application error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "internal_error",
                        message: "unexpected internal server error".to_owned(),
                        owner: None,
                    }),
                )
                    .into_response()
            }
        }
    }
}
