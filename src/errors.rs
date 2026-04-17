use axum::{
    Json,
    http::StatusCode,
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
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    error: "bad_request",
                    message,
                }),
            )
                .into_response(),
            AppError::Config(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: "config_error",
                    message,
                }),
            )
                .into_response(),
            AppError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                Json(ErrorBody {
                    error: "forbidden",
                    message,
                }),
            )
                .into_response(),
            AppError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "not_found",
                    message,
                }),
            )
                .into_response(),
            AppError::Internal(error) => {
                tracing::error!(%error, "unexpected internal application error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "internal_error",
                        message: "unexpected internal server error".to_owned(),
                    }),
                )
                    .into_response()
            }
        }
    }
}
