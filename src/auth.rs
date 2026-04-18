use axum::http::{HeaderMap, header::AUTHORIZATION};

use crate::errors::{AppError, AppResult};

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
