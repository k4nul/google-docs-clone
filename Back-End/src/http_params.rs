use uuid::Uuid;

use crate::errors::{AppError, AppResult};

pub(crate) fn parse_uuid_param(parameter: &str, raw_value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(raw_value).map_err(|_| {
        AppError::BadRequest(format!(
            "{parameter} must be a valid UUID, received `{raw_value}`"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uuid_param_accepts_uuid_values() {
        let value = "00000000-0000-4000-8000-000000000000";

        let parsed = parse_uuid_param("id", value).expect("uuid path parameter should parse");

        assert_eq!(parsed.to_string(), value);
    }

    #[test]
    fn parse_uuid_param_returns_bad_request_with_parameter_name() {
        let error = parse_uuid_param("doc_id", "not-a-uuid")
            .expect_err("invalid uuid path parameter should be rejected");

        assert!(matches!(
            error,
            AppError::BadRequest(message)
                if message == "doc_id must be a valid UUID, received `not-a-uuid`"
        ));
    }
}
