use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub hide_preview: bool,
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
            hide_preview: false,
            access_token: Uuid::new_v4().to_string(),
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn rename(&mut self, title: String) {
        self.title = title;
        self.touch();
    }

    pub fn set_hide_preview(&mut self, hide_preview: bool) {
        if self.hide_preview != hide_preview {
            self.hide_preview = hide_preview;
            self.touch();
        }
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn authorize(&self, token: &str) -> bool {
        crate::secrets::constant_time_eq(&self.access_token, token)
    }

    pub(crate) fn from_parts(
        id: Uuid,
        title: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        access_token: String,
        hide_preview: bool,
    ) -> Self {
        Self {
            id,
            title,
            created_at,
            updated_at,
            hide_preview,
            access_token,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_new_uses_provided_title() {
        let doc = Document::new(Uuid::new_v4(), Some("My Doc".to_owned()));
        assert_eq!(doc.title, "My Doc");
    }

    #[test]
    fn document_new_trims_whitespace_from_title() {
        let doc = Document::new(Uuid::new_v4(), Some("  Trimmed  ".to_owned()));
        assert_eq!(doc.title, "Trimmed");
    }

    #[test]
    fn document_new_assigns_default_title_when_title_is_none() {
        let id = Uuid::new_v4();
        let doc = Document::new(id, None);
        assert_eq!(doc.title, format!("Document {id}"));
    }

    #[test]
    fn document_new_assigns_default_title_when_title_is_empty() {
        let id = Uuid::new_v4();
        let doc = Document::new(id, Some(String::new()));
        assert_eq!(doc.title, format!("Document {id}"));
    }

    #[test]
    fn document_new_assigns_default_title_when_title_is_whitespace_only() {
        let id = Uuid::new_v4();
        let doc = Document::new(id, Some("   ".to_owned()));
        assert_eq!(doc.title, format!("Document {id}"));
    }

    #[test]
    fn document_new_sets_created_at_and_updated_at_to_same_instant() {
        let doc = Document::new(Uuid::new_v4(), Some("Test".to_owned()));
        assert_eq!(doc.created_at, doc.updated_at);
    }

    #[test]
    fn document_access_token_is_not_empty() {
        let doc = Document::new(Uuid::new_v4(), Some("Test".to_owned()));
        assert!(!doc.access_token().is_empty());
    }

    #[test]
    fn document_authorize_returns_true_for_correct_token() {
        let doc = Document::new(Uuid::new_v4(), Some("Test".to_owned()));
        assert!(doc.authorize(doc.access_token()));
    }

    #[test]
    fn document_authorize_returns_false_for_wrong_token() {
        let doc = Document::new(Uuid::new_v4(), Some("Test".to_owned()));
        assert!(!doc.authorize("wrong-token"));
    }

    #[test]
    fn document_touch_advances_updated_at_without_changing_created_at() {
        let mut doc = Document::new(Uuid::new_v4(), Some("Test".to_owned()));
        let original_created_at = doc.created_at;
        let original_updated_at = doc.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(5));
        doc.touch();
        assert_eq!(doc.created_at, original_created_at);
        assert!(doc.updated_at > original_updated_at);
    }

    #[test]
    fn document_access_token_is_omitted_from_serialized_json() {
        let doc = Document::new(Uuid::new_v4(), Some("Test".to_owned()));
        let value = serde_json::to_value(&doc).expect("document should serialize");
        assert!(value.get("access_token").is_none());
    }

    #[test]
    fn document_serializes_preview_visibility_setting() {
        let mut doc = Document::new(Uuid::new_v4(), Some("Test".to_owned()));
        doc.set_hide_preview(true);

        let value = serde_json::to_value(&doc).expect("document should serialize");

        assert_eq!(
            value.get("hide_preview").and_then(|value| value.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn document_two_instances_with_same_id_have_different_access_tokens() {
        let id = Uuid::new_v4();
        let doc_a = Document::new(id, Some("Test".to_owned()));
        let doc_b = Document::new(id, Some("Test".to_owned()));
        assert_ne!(doc_a.access_token(), doc_b.access_token());
    }
}
