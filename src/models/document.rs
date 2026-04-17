use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn placeholder(id: Uuid) -> Self {
        let now = Utc::now();

        Self {
            id,
            title: format!("Document {id}"),
            created_at: now,
            updated_at: now,
        }
    }
}
