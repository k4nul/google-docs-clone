use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use celerix_store::engine::Persistence;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const PERSONA_ID: &str = "snapshots";
const APP_ID: &str = "documents";

pub struct CelerixStoreSnapshotStore {
    root: PathBuf,
    persistence: Persistence,
    write_lock: Mutex<()>,
}

impl CelerixStoreSnapshotStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CELERIX_STORE_PATH cannot be empty when SNAPSHOT_STORE=celerix_store"
                    .to_owned(),
            ));
        }

        let persistence = Persistence::new(&root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", root.display())))?;

        Ok(Self {
            root,
            persistence,
            write_lock: Mutex::new(()),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        doc_id.to_string()
    }

    fn map_error(&self, error: celerix_store::Error) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.root.display()))
    }

    fn load_persona(&self) -> Result<HashMap<String, HashMap<String, Value>>, StorageError> {
        self.persistence
            .load_all()
            .map_err(|error| self.map_error(error))
            .map(|mut data| data.remove(PERSONA_ID).unwrap_or_default())
    }

    fn save_persona(
        &self,
        persona: &HashMap<String, HashMap<String, Value>>,
    ) -> Result<(), StorageError> {
        self.persistence
            .save_persona(PERSONA_ID, persona)
            .map_err(|error| self.map_error(error))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        value: Value,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_value::<PersistedSnapshot>(value)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for CelerixStoreSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let persona = self.load_persona()?;
        let Some(value) = persona
            .get(APP_ID)
            .and_then(|app| app.get(&Self::snapshot_key(doc_id)))
            .cloned()
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_| StorageError::Io("celerix_store write lock was poisoned".to_owned()))?;
        let doc_id = snapshot.document.id;
        let value = serde_json::to_value(PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize celerix_store snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut persona = self.load_persona()?;
        persona
            .entry(APP_ID.to_owned())
            .or_default()
            .insert(Self::snapshot_key(&doc_id), value);
        self.save_persona(&persona)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_| StorageError::Io("celerix_store write lock was poisoned".to_owned()))?;
        let mut persona = self.load_persona()?;
        if let Some(app) = persona.get_mut(APP_ID) {
            app.remove(&Self::snapshot_key(doc_id));
        }
        self.save_persona(&persona)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let persona = self.load_persona()?;
        let Some(app) = persona.get(APP_ID) else {
            return Ok(Vec::new());
        };

        let mut entries = app.iter().collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));

        let mut documents = Vec::new();
        for (key, value) in entries {
            let Ok(doc_id) = Uuid::parse_str(key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value.clone()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.root.display(),
                    "skipping corrupt celerix_store snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
