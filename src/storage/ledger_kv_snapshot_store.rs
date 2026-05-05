use std::{
    fs::{self, File},
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use ledger_kv::{EntryLabel, LedgerKV};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const CATALOG_KEY: &str = "__catalog__";
const LEDGER_DESCRIPTION: &str = "snapshots";

pub struct LedgerKvSnapshotStore {
    path: PathBuf,
    ledger: Mutex<LedgerKV>,
}

impl LedgerKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_LEDGER_KV_PATH cannot be empty when SNAPSHOT_STORE=ledger_kv".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let mut ledger = LedgerKV::new(path.clone(), LEDGER_DESCRIPTION);
        if Self::entry_value(&ledger, CATALOG_KEY.as_bytes()).is_none() {
            ledger
                .upsert(
                    EntryLabel::Unspecified,
                    CATALOG_KEY.as_bytes().to_vec(),
                    b"[]".to_vec(),
                )
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
            Self::sync_files_at(&path)?;
        }

        Ok(Self {
            path,
            ledger: Mutex::new(ledger),
        })
    }

    fn lock_ledger(&self) -> Result<MutexGuard<'_, LedgerKV>, StorageError> {
        self.ledger.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: ledger_kv store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("snapshot:{doc_id}")
    }

    fn entry_value(ledger: &LedgerKV, key: &[u8]) -> Option<Vec<u8>> {
        ledger
            .iter(Some(EntryLabel::Unspecified))
            .find(|entry| entry.key == key)
            .map(|entry| entry.value.clone())
    }

    fn read_value(&self, ledger: &LedgerKV, key: &[u8]) -> Option<Vec<u8>> {
        Self::entry_value(ledger, key)
    }

    fn upsert_value(
        &self,
        ledger: &mut LedgerKV,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        ledger
            .upsert(EntryLabel::Unspecified, key, value)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.sync_files()
    }

    fn delete_value(&self, ledger: &mut LedgerKV, key: Vec<u8>) -> Result<(), StorageError> {
        ledger
            .delete(EntryLabel::Unspecified, key)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.sync_files()
    }

    fn sync_files(&self) -> Result<(), StorageError> {
        Self::sync_files_at(&self.path)
    }

    fn sync_files_at(path: &PathBuf) -> Result<(), StorageError> {
        for file_name in ["snapshots.bin", "snapshots.meta"] {
            let file_path = path.join(file_name);
            if file_path.exists() {
                File::open(&file_path)
                    .and_then(|file| file.sync_all())
                    .map_err(|error| {
                        StorageError::Io(format!("{}: {error}", file_path.display()))
                    })?;
            }
        }

        Ok(())
    }

    fn load_catalog(&self, ledger: &LedgerKV) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.read_value(ledger, CATALOG_KEY.as_bytes()) else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to parse ledger_kv snapshot catalog: {error}",
                self.path.display()
            ))
        })
    }

    fn save_catalog(
        &self,
        ledger: &mut LedgerKV,
        mut catalog: Vec<Uuid>,
    ) -> Result<(), StorageError> {
        catalog.sort_unstable();
        catalog.dedup();
        let payload = serde_json::to_vec(&catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize ledger_kv snapshot catalog: {error}"
            ))
        })?;

        self.upsert_value(ledger, CATALOG_KEY.as_bytes().to_vec(), payload)
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

impl SnapshotStore for LedgerKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let ledger = self.lock_ledger()?;
        let key = Self::snapshot_key(doc_id);
        let Some(payload) = self.read_value(&ledger, key.as_bytes()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut ledger = self.lock_ledger()?;
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize ledger_kv snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.upsert_value(
            &mut ledger,
            Self::snapshot_key(&doc_id).into_bytes(),
            payload,
        )?;
        let mut catalog = self.load_catalog(&ledger)?;
        catalog.push(doc_id);
        self.save_catalog(&mut ledger, catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut ledger = self.lock_ledger()?;
        self.delete_value(&mut ledger, Self::snapshot_key(doc_id).into_bytes())?;
        let mut catalog = self.load_catalog(&ledger)?;
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        self.save_catalog(&mut ledger, catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let ledger = self.lock_ledger()?;
        let catalog = self.load_catalog(&ledger)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let key = Self::snapshot_key(&doc_id);
            let Some(payload) = self.read_value(&ledger, key.as_bytes()) else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing ledger_kv snapshot referenced by catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt ledger_kv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
