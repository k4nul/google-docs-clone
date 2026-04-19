use std::{
    fs,
    io::ErrorKind,
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

        let store = Self { root };
        let cleaned_temp_snapshots = store.cleanup_stale_temp_snapshots()?;
        if cleaned_temp_snapshots > 0 {
            tracing::info!(
                root = %store.root.display(),
                cleaned_temp_snapshots,
                "removed stale temp snapshots during file snapshot store initialization"
            );
        }

        Ok(store)
    }

    fn snapshot_path(&self, doc_id: &Uuid) -> PathBuf {
        self.root.join(format!("{doc_id}.json"))
    }

    fn temp_snapshot_path(&self, doc_id: &Uuid) -> PathBuf {
        self.root
            .join(format!("{doc_id}.json.{}.tmp", Uuid::new_v4()))
    }

    fn temp_snapshot_prefix(&self, doc_id: &Uuid) -> String {
        format!("{doc_id}.json.")
    }

    fn is_temp_snapshot_file_name(file_name: &str) -> bool {
        file_name.ends_with(".tmp") && file_name.contains(".json.")
    }

    fn read_snapshot(&self, path: &Path, doc_id: &Uuid) -> Result<DocumentSnapshot, StorageError> {
        let bytes = fs::read(path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(&bytes)
            .map_err(|_| StorageError::CorruptSnapshot(*doc_id))?;

        Ok(snapshot.into())
    }

    fn remove_file_if_exists(&self, path: &Path) -> Result<(), StorageError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(StorageError::Io(format!("{}: {error}", path.display()))),
        }
    }

    fn stale_temp_snapshot_paths(&self) -> Result<Vec<PathBuf>, StorageError> {
        let mut paths = Vec::new();

        for entry in fs::read_dir(&self.root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.root.display())))?
        {
            let entry = entry.map_err(|error| StorageError::Io(error.to_string()))?;
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            if Self::is_temp_snapshot_file_name(file_name) {
                paths.push(path);
            }
        }

        Ok(paths)
    }

    fn matching_temp_snapshot_paths(&self, doc_id: &Uuid) -> Result<Vec<PathBuf>, StorageError> {
        let temp_prefix = self.temp_snapshot_prefix(doc_id);
        let mut paths = Vec::new();

        for path in self.stale_temp_snapshot_paths()? {
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if file_name.starts_with(&temp_prefix) {
                paths.push(path);
            }
        }

        Ok(paths)
    }

    fn cleanup_stale_temp_snapshots(&self) -> Result<usize, StorageError> {
        let mut removed = 0;

        for path in self.stale_temp_snapshot_paths()? {
            match self.remove_file_if_exists(&path) {
                Ok(()) => removed += 1,
                Err(error) => {
                    tracing::warn!(
                        path = %path.display(),
                        %error,
                        "failed to remove stale temp snapshot during file snapshot store initialization"
                    );
                }
            }
        }

        Ok(removed)
    }

    fn write_snapshot_atomically(
        &self,
        doc_id: &Uuid,
        path: &Path,
        bytes: &[u8],
    ) -> Result<(), StorageError> {
        let temp_path = self.temp_snapshot_path(doc_id);

        if let Err(error) = fs::write(&temp_path, bytes) {
            let _ = fs::remove_file(&temp_path);
            return Err(StorageError::Io(format!(
                "{}: {error}",
                temp_path.display()
            )));
        }

        if let Err(error) = fs::rename(&temp_path, path) {
            let _ = fs::remove_file(&temp_path);
            return Err(StorageError::Io(format!("{}: {error}", path.display())));
        }

        Ok(())
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

        self.write_snapshot_atomically(&doc_id, &path, &bytes)?;
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let path = self.snapshot_path(doc_id);

        self.remove_file_if_exists(&path)?;

        for temp_path in self.matching_temp_snapshot_paths(doc_id)? {
            self.remove_file_if_exists(&temp_path)?;
        }

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
