use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DocumentCredentials {
    pub access_token: String,
}
