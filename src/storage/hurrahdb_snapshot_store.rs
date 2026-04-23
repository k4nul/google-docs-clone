use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use hurrahdb::{AofConfig, Config as HurrahdbConfig, Storage, Type};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

enum HurrahdbCommand {
    Load {
        doc_id: Uuid,
        reply: mpsc::SyncSender<Result<Option<DocumentSnapshot>, StorageError>>,
    },
    Save {
        snapshot: DocumentSnapshot,
        reply: mpsc::SyncSender<Result<(), StorageError>>,
    },
    Delete {
        doc_id: Uuid,
        reply: mpsc::SyncSender<Result<(), StorageError>>,
    },
    List {
        reply: mpsc::SyncSender<Result<Vec<Document>, StorageError>>,
    },
}

pub struct HurrahdbSnapshotStore {
    path: PathBuf,
    commands: mpsc::Sender<HurrahdbCommand>,
}

impl HurrahdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_HURRAHDB_PATH cannot be empty when SNAPSHOT_STORE=hurrahdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let file_name = path
            .to_str()
            .ok_or_else(|| {
                StorageError::Config(
                    "SNAPSHOT_HURRAHDB_PATH must be valid unicode when SNAPSHOT_STORE=hurrahdb"
                        .to_owned(),
                )
            })?
            .to_owned();

        let (command_sender, command_receiver) = mpsc::channel();
        let (init_sender, init_receiver) = mpsc::sync_channel(1);
        let worker_path = path.clone();

        thread::Builder::new()
            .name("hurrahdb-snapshot-store".to_owned())
            .spawn(move || match HurrahdbWorker::new(worker_path, file_name) {
                Ok(worker) => {
                    let _ = init_sender.send(Ok(()));
                    worker.run(command_receiver);
                }
                Err(error) => {
                    let _ = init_sender.send(Err(error));
                }
            })
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to spawn hurrahdb worker: {error}",
                    path.display()
                ))
            })?;

        init_receiver.recv().map_err(|error| {
            StorageError::Io(format!(
                "{}: hurrahdb worker failed to initialize: {error}",
                path.display()
            ))
        })??;

        Ok(Self {
            path,
            commands: command_sender,
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn send_command(&self, command: HurrahdbCommand) -> Result<(), StorageError> {
        self.commands.send(command).map_err(|_| {
            StorageError::Io(format!("{}: hurrahdb worker stopped", self.path.display()))
        })
    }

    fn receive_reply<T>(
        &self,
        receiver: mpsc::Receiver<Result<T, StorageError>>,
    ) -> Result<T, StorageError> {
        receiver.recv().map_err(|_| {
            StorageError::Io(format!(
                "{}: hurrahdb worker stopped before replying",
                self.path.display()
            ))
        })?
    }
}

struct HurrahdbWorker {
    path: PathBuf,
    database: Storage,
}

impl HurrahdbWorker {
    fn new(path: PathBuf, file_name: String) -> Result<Self, StorageError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to create hurrahdb worker runtime: {error}",
                    path.display()
                ))
            })?;
        let _runtime_guard = runtime.enter();

        let database = Storage::new(Some(HurrahdbConfig {
            aof_config: Some(AofConfig {
                sync_time: 60_000,
                file_name,
            }),
            persistance_type: Type::Aof,
        }))
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let worker = Self { path, database };
        worker.sync_file()?;

        Ok(worker)
    }

    fn run(self, command_receiver: mpsc::Receiver<HurrahdbCommand>) {
        for command in command_receiver {
            self.handle(command);
        }
    }

    fn handle(&self, command: HurrahdbCommand) {
        match command {
            HurrahdbCommand::Load { doc_id, reply } => {
                let _ = reply.send(self.load_snapshot(&doc_id));
            }
            HurrahdbCommand::Save { snapshot, reply } => {
                let _ = reply.send(self.save_snapshot(snapshot));
            }
            HurrahdbCommand::Delete { doc_id, reply } => {
                let _ = reply.send(self.delete_snapshot(&doc_id));
            }
            HurrahdbCommand::List { reply } => {
                let _ = reply.send(self.list_documents());
            }
        }
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        self.database
            .get::<Vec<Uuid>>(SNAPSHOT_CATALOG_KEY.to_owned())
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to read hurrahdb snapshot catalog: {error}",
                    self.path.display()
                ))
            })
            .map(|catalog| catalog.unwrap_or_default())
    }

    fn write_catalog(&self, catalog: &[Uuid]) -> Result<(), StorageError> {
        self.database
            .set(SNAPSHOT_CATALOG_KEY.to_owned(), &catalog)
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to write hurrahdb snapshot catalog: {error}",
                    self.path.display()
                ))
            })?;
        self.sync_file()
    }

    fn sync_file(&self) -> Result<(), StorageError> {
        File::open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            sync_parent_dir(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        Ok(())
    }

    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let snapshot = self
            .database
            .get::<PersistedSnapshot>(HurrahdbSnapshotStore::snapshot_key(doc_id))
            .map_err(|_| StorageError::CorruptSnapshot(*doc_id))?;

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
        let persisted = PersistedSnapshot::from(snapshot);
        self.database
            .set(HurrahdbSnapshotStore::snapshot_key(&doc_id), &persisted)
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to write hurrahdb snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;
        self.sync_file()?;

        let mut catalog = self.read_catalog()?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.database
            .del(&HurrahdbSnapshotStore::snapshot_key(doc_id));
        self.sync_file()?;

        let mut catalog = self.read_catalog()?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();

        for doc_id in self.read_catalog()? {
            match self.load_snapshot(&doc_id)? {
                Some(snapshot) => documents.push(snapshot.document),
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing hurrahdb snapshot referenced by catalog"
                ),
            }
        }

        Ok(documents)
    }
}

impl SnapshotStore for HurrahdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let (reply, receiver) = mpsc::sync_channel(1);
        self.send_command(HurrahdbCommand::Load {
            doc_id: *doc_id,
            reply,
        })?;
        self.receive_reply(receiver)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let (reply, receiver) = mpsc::sync_channel(1);
        self.send_command(HurrahdbCommand::Save { snapshot, reply })?;
        self.receive_reply(receiver)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let (reply, receiver) = mpsc::sync_channel(1);
        self.send_command(HurrahdbCommand::Delete {
            doc_id: *doc_id,
            reply,
        })?;
        self.receive_reply(receiver)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let (reply, receiver) = mpsc::sync_channel(1);
        self.send_command(HurrahdbCommand::List { reply })?;
        self.receive_reply(receiver)
    }
}

fn sync_parent_dir(path: &Path) -> io::Result<()> {
    File::open(path).and_then(|file| file.sync_all())
}
