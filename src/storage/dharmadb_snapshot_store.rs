use std::{
    fmt::{Display, Formatter},
    fs,
    path::PathBuf,
    sync::mpsc,
    thread,
};

use dharmadb::{
    dharma::Dharma,
    options::DharmaOpts,
    traits::{Nil, ResourceKey, ResourceValue},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const CATALOG_KEY: &str = "__catalog__";
const TOMBSTONE_VALUE: &str = "__dharmadb_deleted_snapshot__";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
struct DharmaSnapshotKey(String);

impl DharmaSnapshotKey {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Display for DharmaSnapshotKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl ResourceKey for DharmaSnapshotKey {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct DharmaSnapshotValue(String);

impl DharmaSnapshotValue {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Display for DharmaSnapshotValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Nil for DharmaSnapshotValue {
    fn nil() -> Self {
        Self(TOMBSTONE_VALUE.to_owned())
    }
}

impl ResourceValue for DharmaSnapshotValue {}

type DharmaSnapshotDatabase = Dharma<DharmaSnapshotKey, DharmaSnapshotValue>;

enum DharmaRequest {
    Load {
        doc_id: Uuid,
        respond_to: mpsc::SyncSender<Result<Option<DocumentSnapshot>, StorageError>>,
    },
    Save {
        snapshot: DocumentSnapshot,
        respond_to: mpsc::SyncSender<Result<(), StorageError>>,
    },
    Delete {
        doc_id: Uuid,
        respond_to: mpsc::SyncSender<Result<(), StorageError>>,
    },
    List {
        respond_to: mpsc::SyncSender<Result<Vec<Document>, StorageError>>,
    },
}

struct DharmaWorker {
    path: PathBuf,
    database: DharmaSnapshotDatabase,
}

impl DharmaWorker {
    fn new(path: PathBuf) -> Result<Self, StorageError> {
        let options = DharmaOpts {
            path: path.to_string_lossy().into_owned(),
            ..DharmaOpts::default()
        };
        let database = DharmaSnapshotDatabase::create(options.clone()).or_else(|_| {
            DharmaSnapshotDatabase::recover::<DharmaSnapshotKey, DharmaSnapshotValue>(options)
        });
        let database =
            database.map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, database })
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_str::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn load_catalog(&mut self) -> Result<Vec<String>, StorageError> {
        let Some(payload) = self
            .database
            .get(&DharmaSnapshotKey::new(CATALOG_KEY))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_str::<Vec<String>>(&payload.0).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to deserialize dharmadb catalog: {error}",
                self.path.display()
            ))
        })
    }

    fn save_catalog(&mut self, catalog: &[String]) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to serialize dharmadb catalog: {error}",
                self.path.display()
            ))
        })?;

        self.database
            .put(
                DharmaSnapshotKey::new(CATALOG_KEY),
                DharmaSnapshotValue::new(payload),
            )
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn load_snapshot(&mut self, doc_id: Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(payload) = self
            .database
            .get(&DharmaSnapshotKey::new(doc_id.to_string()))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(doc_id, &payload.0).map(Some)
    }

    fn save_snapshot(&mut self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize dharmadb snapshot `{doc_id}`: {error}"
                ))
            })?;

        self.database
            .put(
                DharmaSnapshotKey::new(doc_id.to_string()),
                DharmaSnapshotValue::new(payload),
            )
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        let mut catalog = self.load_catalog()?;
        let doc_id_key = doc_id.to_string();
        if !catalog.iter().any(|entry| entry == &doc_id_key) {
            catalog.push(doc_id_key);
        }
        self.save_catalog(&catalog)?;
        self.flush()
    }

    fn delete_snapshot(&mut self, doc_id: Uuid) -> Result<(), StorageError> {
        self.database
            .delete(DharmaSnapshotKey::new(doc_id.to_string()))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        let doc_id_key = doc_id.to_string();
        let mut catalog = self.load_catalog()?;
        catalog.retain(|entry| entry != &doc_id_key);
        self.save_catalog(&catalog)?;
        self.flush()
    }

    fn list_documents(&mut self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.load_catalog()?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.database.get(&DharmaSnapshotKey::new(doc_id_key)) {
                Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload.0) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt dharmadb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing dharmadb snapshot while building document catalog"
                ),
                Err(error) => {
                    return Err(StorageError::Io(format!(
                        "{}: {error}",
                        self.path.display()
                    )));
                }
            }
        }

        Ok(documents)
    }

    fn flush(&mut self) -> Result<(), StorageError> {
        self.database
            .flush()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

pub struct DharmadbSnapshotStore {
    path: PathBuf,
    requests: mpsc::Sender<DharmaRequest>,
}

impl DharmadbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_DHARMADB_PATH cannot be empty when SNAPSHOT_STORE=dharmadb".to_owned(),
            ));
        }

        fs::create_dir_all(path.join("tables"))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let (requests, request_receiver) = mpsc::channel::<DharmaRequest>();
        let (startup_sender, startup_receiver) = mpsc::sync_channel(1);
        let worker_path = path.clone();

        thread::Builder::new()
            .name("dharmadb-snapshot-store".to_owned())
            .spawn(move || {
                let worker = DharmaWorker::new(worker_path);
                let Ok(mut worker) = worker else {
                    let _ = startup_sender.send(worker.map(|_| ()));
                    return;
                };
                let _ = startup_sender.send(Ok(()));

                while let Ok(request) = request_receiver.recv() {
                    match request {
                        DharmaRequest::Load { doc_id, respond_to } => {
                            let _ = respond_to.send(worker.load_snapshot(doc_id));
                        }
                        DharmaRequest::Save {
                            snapshot,
                            respond_to,
                        } => {
                            let _ = respond_to.send(worker.save_snapshot(snapshot));
                        }
                        DharmaRequest::Delete { doc_id, respond_to } => {
                            let _ = respond_to.send(worker.delete_snapshot(doc_id));
                        }
                        DharmaRequest::List { respond_to } => {
                            let _ = respond_to.send(worker.list_documents());
                        }
                    }
                }
            })
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        startup_receiver
            .recv()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))??;

        Ok(Self { path, requests })
    }

    fn request<T>(
        &self,
        build: impl FnOnce(mpsc::SyncSender<Result<T, StorageError>>) -> DharmaRequest,
    ) -> Result<T, StorageError> {
        let (respond_to, response) = mpsc::sync_channel(1);
        self.requests.send(build(respond_to)).map_err(|_| {
            StorageError::Io(format!(
                "{}: dharmadb worker thread is unavailable",
                self.path.display()
            ))
        })?;

        response.recv().map_err(|error| {
            StorageError::Io(format!(
                "{}: dharmadb worker response failed: {error}",
                self.path.display()
            ))
        })?
    }
}

impl SnapshotStore for DharmadbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        self.request(|respond_to| DharmaRequest::Load {
            doc_id: *doc_id,
            respond_to,
        })
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        self.request(|respond_to| DharmaRequest::Save {
            snapshot,
            respond_to,
        })
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.request(|respond_to| DharmaRequest::Delete {
            doc_id: *doc_id,
            respond_to,
        })
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        self.request(|respond_to| DharmaRequest::List { respond_to })
    }
}
