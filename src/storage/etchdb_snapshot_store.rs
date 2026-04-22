use std::{collections::BTreeMap, path::PathBuf};

use etchdb::{
    Collection, Op, Overlay, Replayable, Store, Transactable, WalBackend, apply_op,
    apply_overlay_btree,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_COLLECTION: u8 = 0;
const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct EtchSnapshotState {
    entries: BTreeMap<String, Vec<u8>>,
}

struct EtchSnapshotTx<'a> {
    entries: Collection<'a, String, Vec<u8>, BTreeMap<String, Vec<u8>>>,
}

struct EtchSnapshotOverlay {
    entries: Overlay<String, Vec<u8>>,
}

impl<'a> EtchSnapshotTx<'a> {
    fn put(&mut self, key: String, value: Vec<u8>) {
        self.entries.put(key, value);
    }

    fn delete(&mut self, key: &str) {
        self.entries.delete(&key.to_owned());
    }
}

impl Replayable for EtchSnapshotState {
    fn apply(&mut self, ops: &[Op]) -> etchdb::Result<()> {
        for op in ops {
            apply_op(&mut self.entries, op)?;
        }

        Ok(())
    }
}

impl Transactable for EtchSnapshotState {
    type Tx<'a> = EtchSnapshotTx<'a>;
    type Overlay = EtchSnapshotOverlay;

    fn begin_tx(&self) -> Self::Tx<'_> {
        EtchSnapshotTx {
            entries: Collection::new(&self.entries, SNAPSHOT_COLLECTION),
        }
    }

    fn finish_tx(tx: Self::Tx<'_>) -> (Vec<Op>, Self::Overlay) {
        let (ops, entries) = tx.entries.into_parts();
        (ops, EtchSnapshotOverlay { entries })
    }

    fn apply_overlay(&mut self, overlay: Self::Overlay) {
        apply_overlay_btree(&mut self.entries, overlay.entries);
    }
}

pub struct EtchdbSnapshotStore {
    path: PathBuf,
    store: Store<EtchSnapshotState, WalBackend<EtchSnapshotState>>,
}

impl EtchdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_ETCHDB_PATH cannot be empty when SNAPSHOT_STORE=etchdb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let store =
            Store::<EtchSnapshotState, WalBackend<EtchSnapshotState>>::open_wal(path.clone())
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, store })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn map_error(&self, operation: &str, error: etchdb::Error) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn read_value(&self, key: &str) -> Option<Vec<u8>> {
        self.store.read().entries.get(key).cloned()
    }

    fn write_values(
        &self,
        values: impl IntoIterator<Item = (String, Vec<u8>)>,
        deletes: impl IntoIterator<Item = String>,
    ) -> Result<(), StorageError> {
        let values: Vec<_> = values.into_iter().collect();
        let deletes: Vec<_> = deletes.into_iter().collect();

        self.store
            .write_durable(|tx| {
                for key in &deletes {
                    tx.delete(key);
                }
                for (key, value) in &values {
                    tx.put(key.clone(), value.clone());
                }
                Ok(())
            })
            .map_err(|error| self.map_error("write etchdb snapshot values", error))
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.read_value(SNAPSHOT_CATALOG_KEY) else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: etchdb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn serialize_catalog(catalog: &[Uuid]) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize etchdb snapshot catalog: {error}"
            ))
        })
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for EtchdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(payload) = self.read_value(&Self::snapshot_key(doc_id)) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize etchdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut catalog = self.read_catalog()?;
        let catalog_payload = if catalog.contains(&doc_id) {
            None
        } else {
            catalog.push(doc_id);
            catalog.sort_unstable();
            Some(Self::serialize_catalog(&catalog)?)
        };

        let mut values = vec![(Self::snapshot_key(&doc_id), payload)];
        if let Some(catalog_payload) = catalog_payload {
            values.push((SNAPSHOT_CATALOG_KEY.to_owned(), catalog_payload));
        }

        self.write_values(values, [])
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut catalog = self.read_catalog()?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);

        let mut values = Vec::new();
        if catalog.len() != original_len {
            values.push((
                SNAPSHOT_CATALOG_KEY.to_owned(),
                Self::serialize_catalog(&catalog)?,
            ));
        }

        self.write_values(values, [Self::snapshot_key(doc_id)])
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.read_catalog()?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.load_snapshot(&doc_id) {
                Ok(Some(snapshot)) => documents.push(snapshot.document),
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing etchdb snapshot referenced by catalog"
                ),
                Err(StorageError::CorruptSnapshot(_)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt etchdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
