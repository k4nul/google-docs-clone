use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use lsm_tree::{AbstractTree, AnyTree, Config as LsmTreeConfig, SeqNo};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const CATALOG_KEY: &[u8] = b"__catalog__";

struct LsmTreeInner {
    tree: AnyTree,
    next_seqno: SeqNo,
}

pub struct LsmTreeSnapshotStore {
    path: PathBuf,
    inner: Mutex<LsmTreeInner>,
}

impl LsmTreeSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_LSM_TREE_PATH cannot be empty when SNAPSHOT_STORE=lsm_tree".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let tree = LsmTreeConfig::new(&path, Default::default(), Default::default())
            .open()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let next_seqno = tree.get_highest_seqno().unwrap_or(0).saturating_add(1);

        Ok(Self {
            path,
            inner: Mutex::new(LsmTreeInner { tree, next_seqno }),
        })
    }

    fn lock_inner(&self) -> Result<MutexGuard<'_, LsmTreeInner>, StorageError> {
        self.inner.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: lsm_tree store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn next_seqno(inner: &mut LsmTreeInner) -> SeqNo {
        let seqno = inner.next_seqno;
        inner.next_seqno = inner.next_seqno.saturating_add(1);
        seqno
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("snapshot:{doc_id}")
    }

    fn load_catalog(&self, inner: &LsmTreeInner) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = inner
            .tree
            .get(CATALOG_KEY, SeqNo::MAX)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to parse lsm_tree snapshot catalog: {error}",
                self.path.display()
            ))
        })
    }

    fn save_catalog(
        &self,
        inner: &mut LsmTreeInner,
        mut catalog: Vec<Uuid>,
    ) -> Result<SeqNo, StorageError> {
        catalog.sort_unstable();
        catalog.dedup();
        let payload = serde_json::to_vec(&catalog).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to serialize lsm_tree snapshot catalog: {error}",
                self.path.display()
            ))
        })?;
        let seqno = Self::next_seqno(inner);
        inner.tree.insert(CATALOG_KEY, payload, seqno);
        Ok(seqno)
    }

    fn flush(&self, inner: &LsmTreeInner, seqno: SeqNo) -> Result<(), StorageError> {
        inner
            .tree
            .flush_active_memtable(seqno)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
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

impl SnapshotStore for LsmTreeSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let inner = self.lock_inner()?;
        let key = Self::snapshot_key(doc_id);
        let Some(payload) = inner
            .tree
            .get(key.as_bytes(), SeqNo::MAX)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut inner = self.lock_inner()?;
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize lsm_tree snapshot `{doc_id}`: {error}"
            ))
        })?;

        let snapshot_seqno = Self::next_seqno(&mut inner);
        inner.tree.insert(
            Self::snapshot_key(&doc_id).as_bytes(),
            payload,
            snapshot_seqno,
        );

        let mut catalog = self.load_catalog(&inner)?;
        catalog.push(doc_id);
        let catalog_seqno = self.save_catalog(&mut inner, catalog)?;
        self.flush(&inner, catalog_seqno)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut inner = self.lock_inner()?;
        let delete_seqno = Self::next_seqno(&mut inner);
        inner
            .tree
            .remove(Self::snapshot_key(doc_id).as_bytes(), delete_seqno);

        let mut catalog = self.load_catalog(&inner)?;
        catalog.retain(|candidate| candidate != doc_id);
        let catalog_seqno = self.save_catalog(&mut inner, catalog)?;
        self.flush(&inner, catalog_seqno)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let inner = self.lock_inner()?;
        let catalog = self.load_catalog(&inner)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let key = Self::snapshot_key(&doc_id);
            let Some(payload) = inner
                .tree
                .get(key.as_bytes(), SeqNo::MAX)
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
            else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing lsm_tree snapshot referenced by catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt lsm_tree snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
