use std::{
    collections::HashMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    collab::coordinator::PersistedRoomCoordinatorState,
    collab::managed::{ManagedCoordinationClient, ManagedCoordinationClientError},
    config::{Config, normalize_origin_url},
    errors::{AppError, AppResult},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedRoom {
    Local,
    Remote(RoomOwnerHint),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RoomOwnerHint {
    pub node_id: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

impl RoomOwnerHint {
    fn normalized(mut self) -> Result<Self, RoomLocatorError> {
        let node_id = self.node_id.trim();
        if node_id.is_empty() {
            return Err(RoomLocatorError::Config(
                "room owner hint node_id cannot be empty".to_owned(),
            ));
        }
        self.node_id = node_id.to_owned();

        if let Some(base_url) = self.base_url.take() {
            let base_url = base_url.trim();
            if base_url.is_empty() {
                return Err(RoomLocatorError::Config(
                    "room owner hint base_url cannot be empty".to_owned(),
                ));
            }

            self.base_url = Some(normalize_owner_base_url(base_url)?);
        }

        Ok(self)
    }
}

fn normalize_owner_base_url(base_url: &str) -> Result<String, RoomLocatorError> {
    normalize_origin_url(base_url, "room owner hint base_url").map_err(RoomLocatorError::Config)
}

fn parse_room_coordinator_timestamp(
    value: &str,
    doc_id: Uuid,
) -> Result<chrono::DateTime<Utc>, RoomLocatorError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| {
            RoomLocatorError::LookupFailed(format!(
                "persisted sqlite coordinator state for document `{doc_id}` contains an invalid RFC3339 timestamp"
            ))
        })
}

fn sqlite_room_coordinator_state_from_row(
    row: SqliteRoomCoordinatorRow,
) -> Result<PersistedRoomCoordinatorState, RoomLocatorError> {
    let doc_id = Uuid::parse_str(&row.doc_id).map_err(|_| {
        RoomLocatorError::LookupFailed(format!(
            "persisted sqlite room lease row contains an invalid doc_id `{}`",
            row.doc_id
        ))
    })?;
    let lease_id = Uuid::parse_str(&row.lease_id).map_err(|_| {
        RoomLocatorError::LookupFailed(format!(
            "persisted sqlite room lease row for document `{doc_id}` contains an invalid lease_id"
        ))
    })?;
    let epoch = u64::try_from(row.epoch).map_err(|_| {
        RoomLocatorError::LookupFailed(format!(
            "persisted sqlite room lease row for document `{doc_id}` contains a negative epoch"
        ))
    })?;

    Ok(PersistedRoomCoordinatorState {
        doc_id,
        node_id: row.node_id,
        base_url: row.base_url,
        lease_id: Some(lease_id),
        epoch,
        activated_at: parse_room_coordinator_timestamp(&row.activated_at, doc_id)?,
        renewed_at: Some(parse_room_coordinator_timestamp(&row.renewed_at, doc_id)?),
        expires_at: Some(parse_room_coordinator_timestamp(&row.expires_at, doc_id)?),
    })
}

#[derive(Debug, Error)]
pub enum RoomLocatorError {
    #[error("room ownership locator is misconfigured: {0}")]
    Config(String),
    #[error("room ownership lookup failed: {0}")]
    LookupFailed(String),
}

pub trait RoomLocator: Send + Sync {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError>;
}

#[derive(Debug, Default)]
pub struct LocalRoomLocator;

impl RoomLocator for LocalRoomLocator {
    fn resolve(&self, _doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        Ok(ResolvedRoom::Local)
    }
}

#[derive(Debug)]
pub struct StaticRoomLocator {
    current_node_id: String,
    document_owners: HashMap<Uuid, RoomOwnerHint>,
}

#[derive(Debug)]
pub struct FileRoomLocator {
    current_node_id: String,
    root: PathBuf,
}

#[derive(Debug)]
pub struct SqliteRoomLocator {
    current_node_id: String,
    path: PathBuf,
}

#[derive(Debug)]
pub struct ManagedRoomLocator {
    current_node_id: String,
    client: ManagedCoordinationClient,
}

#[derive(Debug, Deserialize)]
struct StaticRoomOwnerHints {
    #[serde(default)]
    documents: HashMap<Uuid, RoomOwnerHint>,
}

#[derive(Debug)]
struct SqliteRoomCoordinatorRow {
    doc_id: String,
    node_id: String,
    base_url: Option<String>,
    lease_id: String,
    epoch: i64,
    activated_at: String,
    renewed_at: String,
    expires_at: String,
}

impl StaticRoomLocator {
    pub fn new(
        current_node_id: impl Into<String>,
        document_owners: HashMap<Uuid, RoomOwnerHint>,
    ) -> Result<Self, RoomLocatorError> {
        let current_node_id = current_node_id.into();
        let current_node_id = current_node_id.trim();
        if current_node_id.is_empty() {
            return Err(RoomLocatorError::Config(
                "NODE_ID cannot be empty when ROOM_LOCATOR=static".to_owned(),
            ));
        }

        let document_owners = document_owners
            .into_iter()
            .map(|(doc_id, owner)| owner.normalized().map(|owner| (doc_id, owner)))
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Self {
            current_node_id: current_node_id.to_owned(),
            document_owners,
        })
    }

    pub fn from_json_file(
        current_node_id: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, RoomLocatorError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|error| {
            RoomLocatorError::LookupFailed(format!("{}: {error}", path.display()))
        })?;
        let hints: StaticRoomOwnerHints = serde_json::from_slice(&bytes)
            .map_err(|error| RoomLocatorError::Config(format!("{}: {error}", path.display())))?;

        Self::new(current_node_id, hints.documents)
    }
}

impl FileRoomLocator {
    pub fn new(
        current_node_id: impl Into<String>,
        root: impl Into<PathBuf>,
    ) -> Result<Self, RoomLocatorError> {
        let current_node_id = current_node_id.into();
        let current_node_id = current_node_id.trim();
        if current_node_id.is_empty() {
            return Err(RoomLocatorError::Config(
                "NODE_ID cannot be empty when ROOM_LOCATOR=file".to_owned(),
            ));
        }

        let root = root.into();
        fs::create_dir_all(&root).map_err(|error| {
            RoomLocatorError::LookupFailed(format!(
                "failed to create room coordinator state dir `{}`: {error}",
                root.display()
            ))
        })?;

        Ok(Self {
            current_node_id: current_node_id.to_owned(),
            root,
        })
    }

    fn state_path(&self, doc_id: &Uuid) -> PathBuf {
        self.root.join(format!("{doc_id}.json"))
    }
}

impl SqliteRoomLocator {
    pub fn new(
        current_node_id: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Result<Self, RoomLocatorError> {
        let current_node_id = current_node_id.into();
        let current_node_id = current_node_id.trim();
        if current_node_id.is_empty() {
            return Err(RoomLocatorError::Config(
                "NODE_ID cannot be empty when ROOM_LOCATOR=sqlite".to_owned(),
            ));
        }

        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(RoomLocatorError::Config(
                "ROOM_COORDINATOR_SQLITE_PATH cannot be empty when ROOM_LOCATOR=sqlite".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|error| {
                RoomLocatorError::LookupFailed(format!(
                    "failed to create sqlite room coordinator parent dir `{}`: {error}",
                    parent.display()
                ))
            })?;
        }

        let locator = Self {
            current_node_id: current_node_id.to_owned(),
            path,
        };
        let connection = locator.open_connection()?;
        locator.initialize_schema(&connection)?;
        Ok(locator)
    }

    fn open_connection(&self) -> Result<Connection, RoomLocatorError> {
        let connection = Connection::open(&self.path).map_err(|error| {
            RoomLocatorError::LookupFailed(format!(
                "failed to open sqlite room coordinator database `{}`: {error}",
                self.path.display()
            ))
        })?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|error| {
                RoomLocatorError::LookupFailed(format!(
                    "failed to configure sqlite room coordinator busy timeout `{}`: {error}",
                    self.path.display()
                ))
            })?;
        Ok(connection)
    }

    fn initialize_schema(&self, connection: &Connection) -> Result<(), RoomLocatorError> {
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS room_leases (
                    doc_id TEXT PRIMARY KEY,
                    node_id TEXT NOT NULL,
                    base_url TEXT,
                    lease_id TEXT NOT NULL,
                    epoch INTEGER NOT NULL,
                    activated_at TEXT NOT NULL,
                    renewed_at TEXT NOT NULL,
                    expires_at TEXT NOT NULL
                );",
            )
            .map_err(|error| {
                RoomLocatorError::LookupFailed(format!(
                    "failed to initialize sqlite room coordinator schema `{}`: {error}",
                    self.path.display()
                ))
            })?;
        Ok(())
    }
}

impl ManagedRoomLocator {
    pub fn new(
        current_node_id: impl Into<String>,
        managed_base_url: impl Into<String>,
        managed_auth_token: Option<String>,
        managed_timeout: std::time::Duration,
    ) -> Result<Self, RoomLocatorError> {
        let current_node_id = current_node_id.into();
        let current_node_id = current_node_id.trim();
        if current_node_id.is_empty() {
            return Err(RoomLocatorError::Config(
                "NODE_ID cannot be empty when ROOM_LOCATOR=managed".to_owned(),
            ));
        }

        let client =
            ManagedCoordinationClient::new(managed_base_url, managed_auth_token, managed_timeout)
                .map_err(|error| {
                match error {
            ManagedCoordinationClientError::Config(message)
            | ManagedCoordinationClientError::Request(message) => RoomLocatorError::Config(message),
            ManagedCoordinationClientError::Conflict(_) => RoomLocatorError::Config(
                "managed coordination client returned an unexpected conflict during initialization"
                    .to_owned(),
            ),
        }
            })?;

        Ok(Self {
            current_node_id: current_node_id.to_owned(),
            client,
        })
    }
}

impl RoomLocator for StaticRoomLocator {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        match self.document_owners.get(doc_id) {
            Some(owner) if owner.node_id != self.current_node_id => {
                Ok(ResolvedRoom::Remote(owner.clone()))
            }
            _ => Ok(ResolvedRoom::Local),
        }
    }
}

impl RoomLocator for FileRoomLocator {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        let path = self.state_path(doc_id);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(ResolvedRoom::Local),
            Err(error) => {
                return Err(RoomLocatorError::LookupFailed(format!(
                    "{}: {error}",
                    path.display()
                )));
            }
        };

        let state: PersistedRoomCoordinatorState =
            serde_json::from_slice(&bytes).map_err(|error| {
                RoomLocatorError::LookupFailed(format!("{}: {error}", path.display()))
            })?;

        if state.doc_id != *doc_id {
            return Err(RoomLocatorError::LookupFailed(format!(
                "{}: persisted coordinator state doc_id `{}` did not match requested doc_id `{doc_id}`",
                path.display(),
                state.doc_id
            )));
        }

        let owner_node_id = state.node_id.trim();
        if owner_node_id.is_empty() {
            return Err(RoomLocatorError::LookupFailed(format!(
                "{}: persisted coordinator state node_id cannot be empty",
                path.display()
            )));
        }

        if state
            .expires_at
            .map(|expires_at| expires_at <= Utc::now())
            .unwrap_or(false)
        {
            return Ok(ResolvedRoom::Local);
        }

        let base_url = match state.base_url.as_deref() {
            Some(base_url) => Some(normalize_owner_base_url(base_url).map_err(|error| {
                RoomLocatorError::LookupFailed(format!("{}: {error}", path.display()))
            })?),
            None => None,
        };

        if owner_node_id == self.current_node_id {
            Ok(ResolvedRoom::Local)
        } else {
            Ok(ResolvedRoom::Remote(RoomOwnerHint {
                node_id: owner_node_id.to_owned(),
                base_url,
            }))
        }
    }
}

impl RoomLocator for SqliteRoomLocator {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        let connection = self.open_connection()?;
        self.initialize_schema(&connection)?;
        let row = connection
            .query_row(
                "SELECT doc_id, node_id, base_url, lease_id, epoch, activated_at, renewed_at, expires_at
                 FROM room_leases
                 WHERE doc_id = ?1",
                [doc_id.to_string()],
                |row| {
                    Ok(SqliteRoomCoordinatorRow {
                        doc_id: row.get(0)?,
                        node_id: row.get(1)?,
                        base_url: row.get(2)?,
                        lease_id: row.get(3)?,
                        epoch: row.get(4)?,
                        activated_at: row.get(5)?,
                        renewed_at: row.get(6)?,
                        expires_at: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(|error| {
                RoomLocatorError::LookupFailed(format!(
                    "failed to read sqlite room lease `{}` for document `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;

        let Some(state) = row
            .map(sqlite_room_coordinator_state_from_row)
            .transpose()?
        else {
            return Ok(ResolvedRoom::Local);
        };

        let owner_node_id = state.node_id.trim();
        if owner_node_id.is_empty() {
            return Err(RoomLocatorError::LookupFailed(format!(
                "persisted sqlite coordinator state for document `{doc_id}` has an empty node_id"
            )));
        }

        if state
            .expires_at
            .map(|expires_at| expires_at <= Utc::now())
            .unwrap_or(false)
        {
            return Ok(ResolvedRoom::Local);
        }

        let base_url = match state.base_url.as_deref() {
            Some(base_url) => Some(normalize_owner_base_url(base_url).map_err(|error| {
                RoomLocatorError::LookupFailed(format!(
                    "failed to normalize sqlite room lease owner base_url for document `{doc_id}`: {error}"
                ))
            })?),
            None => None,
        };

        if owner_node_id == self.current_node_id {
            Ok(ResolvedRoom::Local)
        } else {
            Ok(ResolvedRoom::Remote(RoomOwnerHint {
                node_id: owner_node_id.to_owned(),
                base_url,
            }))
        }
    }
}

impl RoomLocator for ManagedRoomLocator {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        let Some(state) = self.client.lookup_lease(doc_id).map_err(|error| match error {
            ManagedCoordinationClientError::Config(message)
            | ManagedCoordinationClientError::Request(message) => {
                RoomLocatorError::LookupFailed(message)
            }
            ManagedCoordinationClientError::Conflict(_) => RoomLocatorError::LookupFailed(
                format!(
                    "managed coordination lookup unexpectedly returned a conflict for document `{doc_id}`"
                ),
            ),
        })? else {
            return Ok(ResolvedRoom::Local);
        };

        let owner_node_id = state.node_id.trim();
        if owner_node_id.is_empty() {
            return Err(RoomLocatorError::LookupFailed(format!(
                "managed coordination lease for document `{doc_id}` has an empty node_id"
            )));
        }

        if state
            .expires_at
            .map(|expires_at| expires_at <= Utc::now())
            .unwrap_or(false)
        {
            return Ok(ResolvedRoom::Local);
        }

        let base_url = match state.base_url.as_deref() {
            Some(base_url) => Some(normalize_owner_base_url(base_url).map_err(|error| {
                RoomLocatorError::LookupFailed(format!(
                    "failed to normalize managed lease owner base_url for document `{doc_id}`: {error}"
                ))
            })?),
            None => None,
        };

        if owner_node_id == self.current_node_id {
            Ok(ResolvedRoom::Local)
        } else {
            Ok(ResolvedRoom::Remote(RoomOwnerHint {
                node_id: owner_node_id.to_owned(),
                base_url,
            }))
        }
    }
}

pub fn local_room_locator() -> Arc<dyn RoomLocator> {
    Arc::new(LocalRoomLocator)
}

pub fn room_locator_from_config(config: &Config) -> AppResult<Arc<dyn RoomLocator>> {
    match config.room_locator.as_str() {
        "local" => Ok(local_room_locator()),
        "static" => {
            let path = config.room_owner_hints_path.as_ref().ok_or_else(|| {
                AppError::Config(
                    "ROOM_OWNER_HINTS_PATH is required when ROOM_LOCATOR=static".to_owned(),
                )
            })?;
            let locator = StaticRoomLocator::from_json_file(config.node_id.clone(), path).map_err(
                |error| match error {
                    RoomLocatorError::Config(message) => AppError::Config(message),
                    other => AppError::from(anyhow::Error::from(other)),
                },
            )?;
            Ok(Arc::new(locator))
        }
        "file" => {
            let locator = FileRoomLocator::new(
                config.node_id.clone(),
                config.room_coordinator_state_dir.clone(),
            )
            .map_err(|error| match error {
                RoomLocatorError::Config(message) => AppError::Config(message),
                other => AppError::from(anyhow::Error::from(other)),
            })?;
            Ok(Arc::new(locator))
        }
        "sqlite" => {
            let locator = SqliteRoomLocator::new(
                config.node_id.clone(),
                config.room_coordinator_sqlite_path.clone(),
            )
            .map_err(|error| match error {
                RoomLocatorError::Config(message) => AppError::Config(message),
                other => AppError::from(anyhow::Error::from(other)),
            })?;
            Ok(Arc::new(locator))
        }
        "managed" => {
            let managed_base_url = config
                .room_coordination_managed_base_url
                .clone()
                .ok_or_else(|| {
                    AppError::Config(
                        "ROOM_COORDINATION_MANAGED_BASE_URL is required when ROOM_LOCATOR=managed"
                            .to_owned(),
                    )
                })?;
            let locator = ManagedRoomLocator::new(
                config.node_id.clone(),
                managed_base_url,
                config.room_coordination_managed_auth_token.clone(),
                std::time::Duration::from_secs(config.room_coordination_managed_timeout_secs),
            )
            .map_err(|error| match error {
                RoomLocatorError::Config(message) => AppError::Config(message),
                other => AppError::from(anyhow::Error::from(other)),
            })?;
            Ok(Arc::new(locator))
        }
        other => Err(AppError::Config(format!(
            "ROOM_LOCATOR must be `local`, `static`, `file`, `sqlite`, or `managed`, received `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_NODE_ID;
    use chrono::Utc;
    use std::{fs, path::PathBuf};

    fn temp_hints_path(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("backend-{test_name}-{}.json", Uuid::new_v4()))
    }

    fn temp_state_dir(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "backend-room-locator-{test_name}-{}",
            Uuid::new_v4()
        ))
    }

    fn temp_sqlite_path(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "backend-room-locator-{test_name}-{}.sqlite3",
            Uuid::new_v4()
        ))
    }

    #[test]
    fn local_room_locator_marks_every_document_as_local() {
        let locator = LocalRoomLocator;
        let doc_id = Uuid::new_v4();

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("local locator should always resolve locally"),
            ResolvedRoom::Local
        );
    }

    #[test]
    fn static_room_locator_marks_other_node_document_as_remote() {
        let doc_id = Uuid::new_v4();
        let locator = StaticRoomLocator::new(
            "node-a",
            HashMap::from([(
                doc_id,
                RoomOwnerHint {
                    node_id: "node-b".to_owned(),
                    base_url: Some("http://node-b.internal:4000".to_owned()),
                },
            )]),
        )
        .expect("static locator should initialize");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through static locator"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: Some("http://node-b.internal:4000".to_owned()),
            })
        );
    }

    #[test]
    fn static_room_locator_marks_current_node_document_as_local() {
        let doc_id = Uuid::new_v4();
        let locator = StaticRoomLocator::new(
            "node-a",
            HashMap::from([(
                doc_id,
                RoomOwnerHint {
                    node_id: "node-a".to_owned(),
                    base_url: Some("http://node-a.internal:4000".to_owned()),
                },
            )]),
        )
        .expect("static locator should initialize");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through static locator"),
            ResolvedRoom::Local
        );
    }

    #[test]
    fn static_room_locator_rejects_invalid_owner_base_url() {
        let doc_id = Uuid::new_v4();
        let error = StaticRoomLocator::new(
            "node-a",
            HashMap::from([(
                doc_id,
                RoomOwnerHint {
                    node_id: "node-b".to_owned(),
                    base_url: Some("node-b.internal:4000".to_owned()),
                },
            )]),
        )
        .expect_err("relative owner base_url should be rejected");

        assert_eq!(
            error.to_string(),
            "room ownership locator is misconfigured: room owner hint base_url must be an origin-only absolute http/https URL without path/query, received `node-b.internal:4000`"
        );
    }

    #[test]
    fn static_room_locator_trims_owner_hint_node_id_and_base_url() {
        let doc_id = Uuid::new_v4();
        let locator = StaticRoomLocator::new(
            " node-a ",
            HashMap::from([(
                doc_id,
                RoomOwnerHint {
                    node_id: "  node-b  ".to_owned(),
                    base_url: Some("  https://node-b.internal:4000/  ".to_owned()),
                },
            )]),
        )
        .expect("static locator should initialize");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through normalized static locator"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: Some("https://node-b.internal:4000".to_owned()),
            })
        );
    }

    #[test]
    fn file_room_locator_marks_other_node_document_as_remote() {
        let doc_id = Uuid::new_v4();
        let root = temp_state_dir("file-remote");
        let locator =
            FileRoomLocator::new("node-a", &root).expect("file locator should initialize");
        let state_path = root.join(format!("{doc_id}.json"));
        let now = Utc::now();

        fs::write(
            &state_path,
            serde_json::to_vec(&PersistedRoomCoordinatorState {
                doc_id,
                node_id: " node-b ".to_owned(),
                base_url: None,
                lease_id: Some(Uuid::new_v4()),
                epoch: 1,
                activated_at: now,
                renewed_at: Some(now),
                expires_at: Some(now + chrono::TimeDelta::seconds(30)),
            })
            .expect("persisted state should serialize"),
        )
        .expect("persisted coordinator state should be written");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("remote coordinator state should resolve"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: None,
            })
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn file_room_locator_marks_current_node_document_as_local() {
        let doc_id = Uuid::new_v4();
        let root = temp_state_dir("file-local");
        let locator =
            FileRoomLocator::new("node-a", &root).expect("file locator should initialize");
        let state_path = root.join(format!("{doc_id}.json"));
        let now = Utc::now();

        fs::write(
            &state_path,
            serde_json::to_vec(&PersistedRoomCoordinatorState {
                doc_id,
                node_id: " node-a ".to_owned(),
                base_url: None,
                lease_id: Some(Uuid::new_v4()),
                epoch: 2,
                activated_at: now,
                renewed_at: Some(now),
                expires_at: Some(now + chrono::TimeDelta::seconds(30)),
            })
            .expect("persisted state should serialize"),
        )
        .expect("persisted coordinator state should be written");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("local coordinator state should resolve"),
            ResolvedRoom::Local
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sqlite_room_locator_marks_other_node_document_as_remote() {
        let doc_id = Uuid::new_v4();
        let path = temp_sqlite_path("sqlite-remote");
        let locator =
            SqliteRoomLocator::new("node-a", &path).expect("sqlite locator should initialize");
        let connection = Connection::open(&path).expect("sqlite file should be writable");
        locator
            .initialize_schema(&connection)
            .expect("sqlite schema should initialize");
        let now = Utc::now();
        connection
            .execute(
                "INSERT INTO room_leases (
                    doc_id,
                    node_id,
                    base_url,
                    lease_id,
                    epoch,
                    activated_at,
                    renewed_at,
                    expires_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    doc_id.to_string(),
                    " node-b ",
                    "http://node-b.internal:4000/",
                    Uuid::new_v4().to_string(),
                    1_i64,
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                    (now + chrono::TimeDelta::seconds(30)).to_rfc3339(),
                ],
            )
            .expect("sqlite room lease should be written");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("remote sqlite coordinator state should resolve"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: Some("http://node-b.internal:4000".to_owned()),
            })
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn sqlite_room_locator_treats_expired_remote_lease_as_local() {
        let doc_id = Uuid::new_v4();
        let path = temp_sqlite_path("sqlite-expired-remote");
        let locator =
            SqliteRoomLocator::new("node-a", &path).expect("sqlite locator should initialize");
        let connection = Connection::open(&path).expect("sqlite file should be writable");
        locator
            .initialize_schema(&connection)
            .expect("sqlite schema should initialize");
        let now = Utc::now();
        connection
            .execute(
                "INSERT INTO room_leases (
                    doc_id,
                    node_id,
                    base_url,
                    lease_id,
                    epoch,
                    activated_at,
                    renewed_at,
                    expires_at
                ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    doc_id.to_string(),
                    "node-b",
                    Uuid::new_v4().to_string(),
                    4_i64,
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                    (now - chrono::TimeDelta::seconds(1)).to_rfc3339(),
                ],
            )
            .expect("expired sqlite room lease should be written");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("expired remote sqlite coordinator state should resolve locally"),
            ResolvedRoom::Local
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn file_room_locator_treats_expired_remote_lease_as_local() {
        let doc_id = Uuid::new_v4();
        let root = temp_state_dir("file-expired-remote");
        let locator =
            FileRoomLocator::new("node-a", &root).expect("file locator should initialize");
        let state_path = root.join(format!("{doc_id}.json"));
        let now = Utc::now();

        fs::write(
            &state_path,
            serde_json::to_vec(&PersistedRoomCoordinatorState {
                doc_id,
                node_id: "node-b".to_owned(),
                base_url: None,
                lease_id: Some(Uuid::new_v4()),
                epoch: 4,
                activated_at: now,
                renewed_at: Some(now),
                expires_at: Some(now - chrono::TimeDelta::seconds(1)),
            })
            .expect("persisted state should serialize"),
        )
        .expect("persisted coordinator state should be written");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("expired remote coordinator state should resolve locally"),
            ResolvedRoom::Local
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn room_locator_from_config_loads_sqlite_room_state() {
        let doc_id = Uuid::new_v4();
        let path = temp_sqlite_path("config-sqlite");
        let connection = Connection::open(&path).expect("sqlite file should be writable");
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS room_leases (
                    doc_id TEXT PRIMARY KEY,
                    node_id TEXT NOT NULL,
                    base_url TEXT,
                    lease_id TEXT NOT NULL,
                    epoch INTEGER NOT NULL,
                    activated_at TEXT NOT NULL,
                    renewed_at TEXT NOT NULL,
                    expires_at TEXT NOT NULL
                );",
            )
            .expect("sqlite schema should initialize");
        let now = Utc::now();
        connection
            .execute(
                "INSERT INTO room_leases (
                    doc_id,
                    node_id,
                    base_url,
                    lease_id,
                    epoch,
                    activated_at,
                    renewed_at,
                    expires_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    doc_id.to_string(),
                    "node-b",
                    "https://node-b.internal:4100/",
                    Uuid::new_v4().to_string(),
                    3_i64,
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                    (now + chrono::TimeDelta::seconds(30)).to_rfc3339(),
                ],
            )
            .expect("sqlite room lease should be written");

        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_s3_endpoint: None,
            snapshot_s3_region: "us-east-1".to_owned(),
            snapshot_s3_bucket: None,
            snapshot_s3_prefix: "snapshots/".to_owned(),
            snapshot_s3_access_key_id: None,
            snapshot_s3_secret_access_key: None,
            snapshot_s3_session_token: None,
            snapshot_s3_timeout_secs: 5,
            snapshot_s3_path_style: true,
            snapshot_managed_base_url: None,
            snapshot_managed_auth_token: None,
            snapshot_managed_timeout_secs: 5,
            room_locator: "sqlite".to_owned(),
            room_coordinator: "noop".to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_sqlite_path: path.display().to_string(),
            room_coordinator_heartbeat_interval_secs: 10,
            room_coordinator_lease_ttl_secs: 30,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: "node-a".to_owned(),
            node_base_url: None,
            room_owner_hints_path: None,
        };

        let locator =
            room_locator_from_config(&config).expect("config should produce a sqlite room locator");
        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through sqlite locator"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: Some("https://node-b.internal:4100".to_owned()),
            })
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn room_locator_from_config_loads_file_room_state() {
        let doc_id = Uuid::new_v4();
        let root = temp_state_dir("config-file");
        let now = Utc::now();
        fs::create_dir_all(&root).expect("test state dir should exist");
        fs::write(
            root.join(format!("{doc_id}.json")),
            serde_json::to_vec(&PersistedRoomCoordinatorState {
                doc_id,
                node_id: "node-b".to_owned(),
                base_url: None,
                lease_id: Some(Uuid::new_v4()),
                epoch: 3,
                activated_at: now,
                renewed_at: Some(now),
                expires_at: Some(now + chrono::TimeDelta::seconds(30)),
            })
            .expect("persisted state should serialize"),
        )
        .expect("persisted coordinator state should be written");

        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_s3_endpoint: None,
            snapshot_s3_region: "us-east-1".to_owned(),
            snapshot_s3_bucket: None,
            snapshot_s3_prefix: "snapshots/".to_owned(),
            snapshot_s3_access_key_id: None,
            snapshot_s3_secret_access_key: None,
            snapshot_s3_session_token: None,
            snapshot_s3_timeout_secs: 5,
            snapshot_s3_path_style: true,
            snapshot_managed_base_url: None,
            snapshot_managed_auth_token: None,
            snapshot_managed_timeout_secs: 5,
            room_locator: "file".to_owned(),
            room_coordinator: "noop".to_owned(),
            room_coordinator_state_dir: root.display().to_string(),
            room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
            room_coordinator_heartbeat_interval_secs: 10,
            room_coordinator_lease_ttl_secs: 30,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: "node-a".to_owned(),
            node_base_url: None,
            room_owner_hints_path: None,
        };

        let locator =
            room_locator_from_config(&config).expect("config should produce a file room locator");
        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through config-backed file locator"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: None,
            })
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn room_locator_from_config_loads_static_owner_hints_file() {
        let doc_id = Uuid::new_v4();
        let hints_path = temp_hints_path("static-room-locator");
        fs::write(
            &hints_path,
            format!(
                r#"{{
  "documents": {{
    "{doc_id}": {{
      "node_id": " node-b ",
      "base_url": " http://node-b.internal:4000/ "
    }}
  }}
}}"#
            ),
        )
        .expect("static room locator hints file should be written");

        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_s3_endpoint: None,
            snapshot_s3_region: "us-east-1".to_owned(),
            snapshot_s3_bucket: None,
            snapshot_s3_prefix: "snapshots/".to_owned(),
            snapshot_s3_access_key_id: None,
            snapshot_s3_secret_access_key: None,
            snapshot_s3_session_token: None,
            snapshot_s3_timeout_secs: 5,
            snapshot_s3_path_style: true,
            snapshot_managed_base_url: None,
            snapshot_managed_auth_token: None,
            snapshot_managed_timeout_secs: 5,
            room_locator: "static".to_owned(),
            room_coordinator: "noop".to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
            room_coordinator_heartbeat_interval_secs: 10,
            room_coordinator_lease_ttl_secs: 30,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: "node-a".to_owned(),
            node_base_url: None,
            room_owner_hints_path: Some(hints_path.to_string_lossy().into_owned()),
        };

        let locator =
            room_locator_from_config(&config).expect("config should produce a static room locator");
        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through config-backed locator"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: Some("http://node-b.internal:4000".to_owned()),
            })
        );

        fs::remove_file(hints_path).expect("static room locator hints file should be removed");
    }

    #[test]
    fn room_locator_from_config_rejects_static_mode_without_hints_path() {
        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_s3_endpoint: None,
            snapshot_s3_region: "us-east-1".to_owned(),
            snapshot_s3_bucket: None,
            snapshot_s3_prefix: "snapshots/".to_owned(),
            snapshot_s3_access_key_id: None,
            snapshot_s3_secret_access_key: None,
            snapshot_s3_session_token: None,
            snapshot_s3_timeout_secs: 5,
            snapshot_s3_path_style: true,
            snapshot_managed_base_url: None,
            snapshot_managed_auth_token: None,
            snapshot_managed_timeout_secs: 5,
            room_locator: "static".to_owned(),
            room_coordinator: "noop".to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
            room_coordinator_heartbeat_interval_secs: 10,
            room_coordinator_lease_ttl_secs: 30,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: "node-a".to_owned(),
            node_base_url: None,
            room_owner_hints_path: None,
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("static locator without hints path should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_OWNER_HINTS_PATH is required when ROOM_LOCATOR=static"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }

    #[test]
    fn room_locator_from_config_rejects_invalid_owner_base_url() {
        let doc_id = Uuid::new_v4();
        let hints_path = temp_hints_path("static-room-locator-invalid-base-url");
        fs::write(
            &hints_path,
            format!(
                r#"{{
  "documents": {{
    "{doc_id}": {{
      "node_id": "node-b",
      "base_url": "ftp://node-b.internal:4000"
    }}
  }}
}}"#
            ),
        )
        .expect("static room locator hints file should be written");

        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_s3_endpoint: None,
            snapshot_s3_region: "us-east-1".to_owned(),
            snapshot_s3_bucket: None,
            snapshot_s3_prefix: "snapshots/".to_owned(),
            snapshot_s3_access_key_id: None,
            snapshot_s3_secret_access_key: None,
            snapshot_s3_session_token: None,
            snapshot_s3_timeout_secs: 5,
            snapshot_s3_path_style: true,
            snapshot_managed_base_url: None,
            snapshot_managed_auth_token: None,
            snapshot_managed_timeout_secs: 5,
            room_locator: "static".to_owned(),
            room_coordinator: "noop".to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
            room_coordinator_heartbeat_interval_secs: 10,
            room_coordinator_lease_ttl_secs: 30,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: "node-a".to_owned(),
            node_base_url: None,
            room_owner_hints_path: Some(hints_path.to_string_lossy().into_owned()),
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("invalid owner base_url should fail config loading"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "room owner hint base_url must be an origin-only absolute http/https URL without path/query, received `ftp://node-b.internal:4000`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }

        fs::remove_file(hints_path).expect("static room locator hints file should be removed");
    }

    #[test]
    fn room_locator_from_config_rejects_owner_base_url_with_path_or_query() {
        let doc_id = Uuid::new_v4();
        let hints_path = temp_hints_path("static-room-locator-invalid-base-url-path");
        fs::write(
            &hints_path,
            format!(
                r#"{{
  "documents": {{
    "{doc_id}": {{
      "node_id": "node-b",
      "base_url": "https://node-b.internal:4000/proxy?via=edge"
    }}
  }}
}}"#
            ),
        )
        .expect("static room locator hints file should be written");

        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_s3_endpoint: None,
            snapshot_s3_region: "us-east-1".to_owned(),
            snapshot_s3_bucket: None,
            snapshot_s3_prefix: "snapshots/".to_owned(),
            snapshot_s3_access_key_id: None,
            snapshot_s3_secret_access_key: None,
            snapshot_s3_session_token: None,
            snapshot_s3_timeout_secs: 5,
            snapshot_s3_path_style: true,
            snapshot_managed_base_url: None,
            snapshot_managed_auth_token: None,
            snapshot_managed_timeout_secs: 5,
            room_locator: "static".to_owned(),
            room_coordinator: "noop".to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
            room_coordinator_heartbeat_interval_secs: 10,
            room_coordinator_lease_ttl_secs: 30,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: "node-a".to_owned(),
            node_base_url: None,
            room_owner_hints_path: Some(hints_path.to_string_lossy().into_owned()),
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("owner base_url with path/query should fail config loading"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "room owner hint base_url must be an origin-only absolute http/https URL without path/query, received `https://node-b.internal:4000/proxy?via=edge`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }

        fs::remove_file(hints_path).expect("static room locator hints file should be removed");
    }

    #[test]
    fn room_locator_from_config_rejects_managed_mode_without_base_url() {
        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_s3_endpoint: None,
            snapshot_s3_region: "us-east-1".to_owned(),
            snapshot_s3_bucket: None,
            snapshot_s3_prefix: "snapshots/".to_owned(),
            snapshot_s3_access_key_id: None,
            snapshot_s3_secret_access_key: None,
            snapshot_s3_session_token: None,
            snapshot_s3_timeout_secs: 5,
            snapshot_s3_path_style: true,
            snapshot_managed_base_url: None,
            snapshot_managed_auth_token: None,
            snapshot_managed_timeout_secs: 5,
            room_locator: "managed".to_owned(),
            room_coordinator: "noop".to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
            room_coordinator_heartbeat_interval_secs: 10,
            room_coordinator_lease_ttl_secs: 30,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: DEFAULT_NODE_ID.to_owned(),
            node_base_url: None,
            room_owner_hints_path: None,
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("managed locator without base url should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_COORDINATION_MANAGED_BASE_URL is required when ROOM_LOCATOR=managed"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }

    #[test]
    fn room_locator_from_config_rejects_unknown_locator_mode() {
        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_s3_endpoint: None,
            snapshot_s3_region: "us-east-1".to_owned(),
            snapshot_s3_bucket: None,
            snapshot_s3_prefix: "snapshots/".to_owned(),
            snapshot_s3_access_key_id: None,
            snapshot_s3_secret_access_key: None,
            snapshot_s3_session_token: None,
            snapshot_s3_timeout_secs: 5,
            snapshot_s3_path_style: true,
            snapshot_managed_base_url: None,
            snapshot_managed_auth_token: None,
            snapshot_managed_timeout_secs: 5,
            room_locator: "unsupported".to_owned(),
            room_coordinator: "noop".to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
            room_coordinator_heartbeat_interval_secs: 10,
            room_coordinator_lease_ttl_secs: 30,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: DEFAULT_NODE_ID.to_owned(),
            node_base_url: None,
            room_owner_hints_path: None,
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("unknown room locator mode should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_LOCATOR must be `local`, `static`, `file`, `sqlite`, or `managed`, received `unsupported`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }
}
