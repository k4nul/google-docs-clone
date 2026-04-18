use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing, skip_deserializing)]
    access_token: String,
}

impl Document {
    pub fn new(id: Uuid, title: Option<String>) -> Self {
        let now = Utc::now();
        let title = title
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("Document {id}"));

        Self {
            id,
            title,
            created_at: now,
            updated_at: now,
            access_token: Uuid::new_v4().to_string(),
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn authorize(&self, token: &str) -> bool {
        self.access_token == token
    }
}
