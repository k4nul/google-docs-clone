use axum::http::{HeaderMap, header::AUTHORIZATION};
use uuid::Uuid;

use crate::{
    collab::rooms::Room,
    errors::{AppError, AppResult},
};

pub fn require_bearer_token(headers: &HeaderMap) -> AppResult<&str> {
    let header = headers
        .get(AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("Authorization header is required".to_owned()))?;

    let value = header.to_str().map_err(|_| {
        AppError::Unauthorized("Authorization header must be valid ASCII".to_owned())
    })?;

    value
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| {
            AppError::Unauthorized("Authorization header must use Bearer token format".to_owned())
        })
}

pub fn require_admin_token(headers: &HeaderMap, expected_token: &str) -> AppResult<()> {
    let token = require_bearer_token(headers)?;

    if !crate::secrets::constant_time_eq(token, expected_token) {
        return Err(AppError::Forbidden(
            "provided API token does not grant this operation".to_owned(),
        ));
    }

    Ok(())
}

pub fn require_document_access(headers: &HeaderMap, room: &Room, doc_id: Uuid) -> AppResult<()> {
    let token = require_bearer_token(headers)?;
    authorize_document_token(token, room, doc_id)
}

pub fn authorize_document_token(token: &str, room: &Room, doc_id: Uuid) -> AppResult<()> {
    if room.authorizes(token) {
        return Ok(());
    }

    Err(AppError::Forbidden(format!(
        "provided token does not grant access to document `{doc_id}`"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers_with(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(value).expect("test header value should be valid"),
        );
        headers
    }

    #[test]
    fn require_bearer_token_returns_token_for_valid_header() {
        let headers = headers_with("Bearer my-secret-token");
        assert_eq!(
            require_bearer_token(&headers).expect("valid bearer token should be returned"),
            "my-secret-token"
        );
    }

    #[test]
    fn require_bearer_token_trims_trailing_whitespace_from_token() {
        let headers = headers_with("Bearer my-token  ");
        assert_eq!(
            require_bearer_token(&headers)
                .expect("token with trailing whitespace should be trimmed"),
            "my-token"
        );
    }

    #[test]
    fn require_bearer_token_rejects_missing_authorization_header() {
        let headers = HeaderMap::new();
        let error = require_bearer_token(&headers).expect_err("missing header should be rejected");
        assert!(matches!(error, AppError::Unauthorized(_)));
    }

    #[test]
    fn require_bearer_token_rejects_non_bearer_scheme() {
        let headers = headers_with("Basic dXNlcjpwYXNz");
        let error =
            require_bearer_token(&headers).expect_err("non-bearer scheme should be rejected");
        assert!(matches!(error, AppError::Unauthorized(_)));
    }

    #[test]
    fn require_bearer_token_rejects_empty_token_after_bearer_prefix() {
        let headers = headers_with("Bearer ");
        let error = require_bearer_token(&headers).expect_err("empty token should be rejected");
        assert!(matches!(error, AppError::Unauthorized(_)));
    }

    #[test]
    fn require_bearer_token_rejects_whitespace_only_token() {
        let headers = headers_with("Bearer    ");
        let error =
            require_bearer_token(&headers).expect_err("whitespace-only token should be rejected");
        assert!(matches!(error, AppError::Unauthorized(_)));
    }

    #[test]
    fn require_bearer_token_rejects_bearer_prefix_without_space() {
        let headers = headers_with("Bearertoken");
        let error = require_bearer_token(&headers)
            .expect_err("bearer without space separator should be rejected");
        assert!(matches!(error, AppError::Unauthorized(_)));
    }
}
