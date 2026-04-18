use std::{
    fs,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

pub struct FileSnapshotStore {
    root: PathBuf,
}

impl FileSnapshotStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        ensure_snapshot_dir(&root)?;

        Ok(Self { root })
    }

    fn snapshot_path(&self, doc_id: &Uuid) -> PathBuf {
        self.root.join(format!("{doc_id}.json"))
    }

    fn read_snapshot(&self, path: &Path, doc_id: &Uuid) -> Result<DocumentSnapshot, StorageError> {
        let bytes = fs::read(path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(&bytes)
            .map_err(|_| StorageError::CorruptSnapshot(*doc_id))?;

        Ok(snapshot.into())
    }
}

impl SnapshotStore for FileSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let path = self.snapshot_path(doc_id);
        if !path.exists() {
            return Ok(None);
        }

        self.read_snapshot(&path, doc_id).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let path = self.snapshot_path(&doc_id);
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        fs::write(&path, bytes)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let path = self.snapshot_path(doc_id);
        if !path.exists() {
            return Ok(());
        }

        fs::remove_file(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();

        for entry in fs::read_dir(&self.root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.root.display())))?
        {
            let entry = entry.map_err(|error| StorageError::Io(error.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(stem) else {
                continue;
            };

            match self.read_snapshot(&path, &doc_id) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(_)) => {
                    tracing::warn!(
                        doc_id = %doc_id,
                        path = %path.display(),
                        "skipping corrupt snapshot while building file-backed document catalog"
                    );
                }
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
