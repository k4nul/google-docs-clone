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
}
