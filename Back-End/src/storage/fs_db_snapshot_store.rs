use std::{
    fs::File,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use fs_db::{Error as FsDbError, FileStore};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_FILE_PREFIX: &str = "snapshot-";
const SNAPSHOT_FILE_SUFFIX: &str = ".json";

pub struct FsDbSnapshotStore {
    root: PathBuf,
    store: Mutex<FileStore<PersistedSnapshot>>,
}

impl FsDbSnapshotStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_FS_DB_PATH cannot be empty when SNAPSHOT_STORE=fs_db".to_owned(),
            ));
        }

        ensure_snapshot_dir(&root)?;
        let store = FileStore::new(&root).map_err(|error| map_fs_db_error(&root, error))?;

        Ok(Self {
            root,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, FileStore<PersistedSnapshot>>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!("{}: fs-db mutex was poisoned", self.root.display()))
        })
    }

    fn snapshot_file_name(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_FILE_PREFIX}{doc_id}{SNAPSHOT_FILE_SUFFIX}")
    }

    fn snapshot_path(&self, doc_id: &Uuid) -> PathBuf {
        self.root.join(Self::snapshot_file_name(doc_id))
    }

    fn parse_doc_id(file_name: &str) -> Option<Uuid> {
        let doc_id = file_name.strip_prefix(SNAPSHOT_FILE_PREFIX)?;
        let doc_id = doc_id.strip_suffix(SNAPSHOT_FILE_SUFFIX)?;
        Uuid::parse_str(doc_id).ok()
    }

    fn sync_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        File::open(self.snapshot_path(doc_id))
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.root.display())))?;
        self.sync_root()
    }

    fn sync_root(&self) -> Result<(), StorageError> {
        sync_dir(&self.root).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to sync fs-db snapshot directory: {error}",
                self.root.display()
            ))
        })
    }

    fn validate_snapshot(
        &self,
        expected_doc_id: Uuid,
        snapshot: PersistedSnapshot,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for FsDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut store = self.lock_store()?;
        let file_name = Self::snapshot_file_name(doc_id);

        match store.load(&file_name) {
            Ok(snapshot) => self.validate_snapshot(*doc_id, snapshot).map(Some),
            Err(FsDbError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(FsDbError::Inner(_)) => Err(StorageError::CorruptSnapshot(*doc_id)),
            Err(error) => Err(map_fs_db_error(&self.root, error)),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let persisted = PersistedSnapshot::from(snapshot);
        let file_name = Self::snapshot_file_name(&doc_id);
        let mut store = self.lock_store()?;

        store
            .store(&file_name, &persisted)
            .map_err(|error| map_fs_db_error(&self.root, error))?;
        drop(store);

        self.sync_snapshot(&doc_id)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let file_name = Self::snapshot_file_name(doc_id);
        let mut store = self.lock_store()?;

        match store.rm(&file_name) {
            Ok(()) => {
                drop(store);
                self.sync_root()
            }
            Err(FsDbError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(map_fs_db_error(&self.root, error)),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut store = self.lock_store()?;
        let mut file_names = store
            .list()
            .map_err(|error| map_fs_db_error(&self.root, error))?;
        file_names.sort();

        let mut documents = Vec::new();
        for file_name in file_names {
            let Some(doc_id) = Self::parse_doc_id(&file_name) else {
                continue;
            };

            match store.load(&file_name) {
                Ok(snapshot) => match self.validate_snapshot(doc_id, snapshot) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.root.display(),
                        "skipping corrupt fs-db snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Err(FsDbError::Io(error)) if error.kind() == ErrorKind::NotFound => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.root.display(),
                    "skipping missing fs-db snapshot while building document catalog"
                ),
                Err(FsDbError::Inner(_)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.root.display(),
                    "skipping undecodable fs-db snapshot while building document catalog"
                ),
                Err(error) => return Err(map_fs_db_error(&self.root, error)),
            }
        }

        Ok(documents)
    }
}

fn map_fs_db_error(path: &Path, error: FsDbError<serde_json::Error>) -> StorageError {
    match error {
        FsDbError::Io(error) => StorageError::Io(format!("{}: {error}", path.display())),
        FsDbError::Inner(error) => StorageError::Io(format!("{}: {error}", path.display())),
    }
}

fn sync_dir(path: &Path) -> io::Result<()> {
    File::open(path).and_then(|file| file.sync_all())
}
