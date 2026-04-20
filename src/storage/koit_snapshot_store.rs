use std::{collections::HashMap, fs, future::Future, path::PathBuf};

use koit::{FileDatabase, format::Json};
use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder, Runtime};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

#[derive(Debug, Default, Serialize, Deserialize)]
struct KoitCatalog {
    snapshots: HashMap<String, PersistedSnapshot>,
}

type KoitDatabase = FileDatabase<KoitCatalog, Json>;

pub struct KoitSnapshotStore {
    path: PathBuf,
    runtime: Runtime,
    database: KoitDatabase,
}

impl KoitSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_KOIT_PATH cannot be empty when SNAPSHOT_STORE=koit".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .thread_name("koit-snapshot-store")
            .build()
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to start koit runtime: {error}",
                    path.display()
                ))
            })?;
        let database = Self::run_with_runtime(
            &runtime,
            FileDatabase::<KoitCatalog, Json>::load_from_path_or_default(&path),
        )
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            runtime,
            database,
        })
    }

    fn run_with_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| runtime.block_on(future))
        } else {
            runtime.block_on(future)
        }
    }

    fn run<T>(&self, future: impl Future<Output = T>) -> T {
        Self::run_with_runtime(&self.runtime, future)
    }
}

impl SnapshotStore for KoitSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let snapshot = self.run(
            self.database
                .read(|catalog| catalog.snapshots.get(&doc_id.to_string()).cloned()),
        );
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != *doc_id {
            return Err(StorageError::CorruptSnapshot(*doc_id));
        }

        Ok(Some(snapshot))
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        self.run(self.database.write(|catalog| {
            catalog
                .snapshots
                .insert(doc_id.to_string(), PersistedSnapshot::from(snapshot));
        }));
        self.run(self.database.save())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.run(self.database.write(|catalog| {
            catalog.snapshots.remove(&doc_id.to_string());
        }));
        self.run(self.database.save())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let snapshots = self.run(self.database.read(|catalog| catalog.snapshots.clone()));
        let mut documents = Vec::new();

        for (doc_id_key, snapshot) in snapshots {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            let snapshot: DocumentSnapshot = snapshot.into();
            if snapshot.document.id != doc_id {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt koit snapshot while building document catalog"
                );
                continue;
            }

            documents.push(snapshot.document);
        }

        Ok(documents)
    }
}
