use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessState {
    pub user: AwarenessUser,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<AwarenessSelection>,
    pub client: AwarenessClient,
}

impl AwarenessState {
    pub fn validate(&self) -> Result<(), AwarenessValidationError> {
        self.user.validate()?;
        self.client.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessUser {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl AwarenessUser {
    fn validate(&self) -> Result<(), AwarenessValidationError> {
        ensure_non_empty("user.id", &self.id)?;
        ensure_non_empty("user.name", &self.name)?;

        if !is_valid_hex_color(&self.color) {
            return Err(AwarenessValidationError::new(
                "user.color",
                "must be a 7-character hex color like `#1f6feb`",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessSelection {
    pub anchor: u32,
    pub head: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessClient {
    pub id: String,
    pub kind: String,
}

impl AwarenessClient {
    fn validate(&self) -> Result<(), AwarenessValidationError> {
        ensure_non_empty("client.id", &self.id)?;
        ensure_non_empty("client.kind", &self.kind)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwarenessValidationError {
    field: &'static str,
    message: &'static str,
}

impl AwarenessValidationError {
    fn new(field: &'static str, message: &'static str) -> Self {
        Self { field, message }
    }

    pub fn field(&self) -> &'static str {
        self.field
    }

    pub fn message(&self) -> &'static str {
        self.message
    }
}

impl std::fmt::Display for AwarenessValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} {}", self.field, self.message)
    }
}

impl std::error::Error for AwarenessValidationError {}

fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), AwarenessValidationError> {
    if value.trim().is_empty() {
        return Err(AwarenessValidationError::new(field, "must not be empty"));
    }

    Ok(())
}

fn is_valid_hex_color(value: &str) -> bool {
    let Some(rest) = value.strip_prefix('#') else {
        return false;
    };

    rest.len() == 6 && rest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awareness_state_serializes_to_frontend_friendly_shape() {
        let state = AwarenessState {
            user: AwarenessUser {
                id: "user-7".to_owned(),
                name: "Kim".to_owned(),
                color: "#1f6feb".to_owned(),
            },
            selection: Some(AwarenessSelection {
                anchor: 3,
                head: 11,
            }),
            client: AwarenessClient {
                id: "session-3".to_owned(),
                kind: "editor".to_owned(),
            },
        };

        let value = serde_json::to_value(&state).expect("awareness state should serialize");

        assert_eq!(value["user"]["id"], "user-7");
        assert_eq!(value["user"]["name"], "Kim");
        assert_eq!(value["user"]["color"], "#1f6feb");
        assert_eq!(value["selection"]["anchor"], 3);
        assert_eq!(value["selection"]["head"], 11);
        assert_eq!(value["client"]["id"], "session-3");
        assert_eq!(value["client"]["kind"], "editor");
    }

    #[test]
    fn awareness_state_validation_rejects_invalid_color() {
        let state = AwarenessState {
            user: AwarenessUser {
                id: "user-7".to_owned(),
                name: "Kim".to_owned(),
                color: "blue".to_owned(),
            },
            selection: None,
            client: AwarenessClient {
                id: "session-3".to_owned(),
                kind: "editor".to_owned(),
            },
        };

        let error = state
            .validate()
            .expect_err("invalid awareness color should be rejected");

        assert_eq!(error.field(), "user.color");
        assert_eq!(
            error.message(),
            "must be a 7-character hex color like `#1f6feb`"
        );
    }

    #[test]
    fn awareness_state_validation_rejects_blank_client_kind() {
        let state = AwarenessState {
            user: AwarenessUser {
                id: "user-7".to_owned(),
                name: "Kim".to_owned(),
                color: "#1f6feb".to_owned(),
            },
            selection: None,
            client: AwarenessClient {
                id: "session-3".to_owned(),
                kind: "   ".to_owned(),
            },
        };

        let error = state
            .validate()
            .expect_err("blank client kind should be rejected");

        assert_eq!(error.field(), "client.kind");
        assert_eq!(error.message(), "must not be empty");
    }
}
