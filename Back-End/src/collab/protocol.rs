use crate::models::awareness::AwarenessState;
use yrs::sync::{
    Awareness, AwarenessUpdate, DefaultProtocol, Error as SyncError, Message, Protocol,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ValidatingProtocol;

impl Protocol for ValidatingProtocol {
    fn handle_awareness_update(
        &self,
        awareness: &mut Awareness,
        update: AwarenessUpdate,
    ) -> Result<Option<Message>, SyncError> {
        validate_awareness_update(&update)?;
        DefaultProtocol.handle_awareness_update(awareness, update)
    }
}

fn validate_awareness_update(update: &AwarenessUpdate) -> Result<(), SyncError> {
    for entry in update.clients.values() {
        if entry.json == "null" {
            continue;
        }

        let state: AwarenessState =
            serde_json::from_str(&entry.json).map_err(|error| SyncError::PermissionDenied {
                reason: format!("invalid awareness payload JSON: {error}"),
            })?;

        state
            .validate()
            .map_err(|error| SyncError::PermissionDenied {
                reason: format!("invalid awareness payload: {error}"),
            })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use yrs::{
        Doc,
        sync::{AwarenessUpdate, awareness::AwarenessUpdateEntry},
    };

    #[test]
    fn validating_protocol_rejects_invalid_awareness_shape() {
        let protocol = ValidatingProtocol;
        let mut awareness = Awareness::new(Doc::new());
        let update = AwarenessUpdate {
            clients: HashMap::from([(
                7,
                AwarenessUpdateEntry {
                    clock: 1,
                    json: r#"{"user":{"id":"user-7","name":"Kim","color":"blue"},"client":{"id":"session-3","kind":"editor"}}"#.to_owned(),
                },
            )]),
        };

        let error = protocol
            .handle_awareness_update(&mut awareness, update)
            .expect_err("invalid awareness payload should be rejected");

        assert!(matches!(error, SyncError::PermissionDenied { .. }));
        assert!(awareness.clients().is_empty());
    }

    #[test]
    fn validating_protocol_accepts_tiptap_collaboration_caret_awareness() {
        let protocol = ValidatingProtocol;
        let mut awareness = Awareness::new(Doc::new());
        let update = AwarenessUpdate {
            clients: HashMap::from([(
                7,
                AwarenessUpdateEntry {
                    clock: 1,
                    json: r##"{
                        "user": {
                            "id": "user-7",
                            "name": "Kim",
                            "color": "#1f6feb"
                        },
                        "cursor": {
                            "anchor": {"type": null, "tname": "content", "item": null, "assoc": 0},
                            "head": {"type": null, "tname": "content", "item": null, "assoc": 0}
                        }
                    }"##
                    .to_owned(),
                },
            )]),
        };

        protocol
            .handle_awareness_update(&mut awareness, update)
            .expect("Tiptap awareness payload should be accepted");

        assert!(awareness.clients().contains_key(&7));
    }

    #[test]
    fn validating_protocol_accepts_empty_y_websocket_awareness() {
        let protocol = ValidatingProtocol;
        let mut awareness = Awareness::new(Doc::new());
        let update = AwarenessUpdate {
            clients: HashMap::from([(
                7,
                AwarenessUpdateEntry {
                    clock: 1,
                    json: "{}".to_owned(),
                },
            )]),
        };

        protocol
            .handle_awareness_update(&mut awareness, update)
            .expect("empty y-websocket awareness payload should be accepted");

        assert!(awareness.clients().contains_key(&7));
    }

    #[test]
    fn validating_protocol_allows_disconnect_markers() {
        let protocol = ValidatingProtocol;
        let mut awareness = Awareness::new(Doc::new());
        let update = AwarenessUpdate {
            clients: HashMap::from([(
                7,
                AwarenessUpdateEntry {
                    clock: 1,
                    json: "null".to_owned(),
                },
            )]),
        };

        protocol
            .handle_awareness_update(&mut awareness, update)
            .expect("disconnect markers should be accepted");
    }

    #[test]
    fn validating_protocol_accepts_valid_awareness_payload() {
        let protocol = ValidatingProtocol;
        let mut awareness = Awareness::new(Doc::new());
        let update = AwarenessUpdate {
            clients: HashMap::from([(
                42,
                AwarenessUpdateEntry {
                    clock: 1,
                    json: r##"{"user":{"id":"user-1","name":"Alice","color":"#1f6feb"},"client":{"id":"session-1","kind":"editor"}}"##.to_owned(),
                },
            )]),
        };

        protocol
            .handle_awareness_update(&mut awareness, update)
            .expect("valid awareness payload should be accepted");

        assert!(awareness.clients().contains_key(&42));
    }

    #[test]
    fn validating_protocol_accepts_valid_awareness_with_selection() {
        let protocol = ValidatingProtocol;
        let mut awareness = Awareness::new(Doc::new());
        let update = AwarenessUpdate {
            clients: HashMap::from([(
                5,
                AwarenessUpdateEntry {
                    clock: 1,
                    json: r##"{"user":{"id":"user-5","name":"Bob","color":"#ff0000"},"selection":{"anchor":3,"head":10},"client":{"id":"session-5","kind":"viewer"}}"##.to_owned(),
                },
            )]),
        };

        protocol
            .handle_awareness_update(&mut awareness, update)
            .expect("valid awareness with selection field should be accepted");

        assert!(awareness.clients().contains_key(&5));
    }

    #[test]
    fn validating_protocol_rejects_malformed_json() {
        let protocol = ValidatingProtocol;
        let mut awareness = Awareness::new(Doc::new());
        let update = AwarenessUpdate {
            clients: HashMap::from([(
                1,
                AwarenessUpdateEntry {
                    clock: 1,
                    json: "not-valid-json".to_owned(),
                },
            )]),
        };

        let error = protocol
            .handle_awareness_update(&mut awareness, update)
            .expect_err("malformed JSON should be rejected");

        assert!(matches!(error, SyncError::PermissionDenied { .. }));
        assert!(awareness.clients().is_empty());
    }
}
