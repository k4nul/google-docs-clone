use std::{
    collections::HashMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::{DateTime, TimeDelta, Utc};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    collab::managed::{ManagedCoordinationClient, ManagedCoordinationClientError},
    config::{Config, normalize_origin_url},
    errors::{AppError, AppResult},
};

#[derive(Debug, Error)]
pub enum RoomCoordinatorError {
    #[error("room coordination failed: {0}")]
    Operation(String),
}

const SQLITE_ROOM_COORDINATOR_BUSY_TIMEOUT_SECS: u64 = 5;

pub trait RoomCoordinator: Send + Sync {
    fn mode(&self) -> &'static str;

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError>;

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError>;
}

#[derive(Debug, Default)]
pub struct NoopRoomCoordinator;

impl RoomCoordinator for NoopRoomCoordinator {
    fn mode(&self) -> &'static str {
        "noop"
    }

    fn room_activated(&self, _doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        Ok(())
    }

    fn room_deactivated(&self, _doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LoggingRoomCoordinator {
    node_id: Arc<str>,
}

impl LoggingRoomCoordinator {
    pub fn new(node_id: impl Into<String>) -> Self {
        let node_id = node_id.into();
        let node_id = node_id.trim();
        let node_id = if node_id.is_empty() {
            "<unknown-node>"
        } else {
            node_id
        };

        Self {
            node_id: Arc::<str>::from(node_id.to_owned()),
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

impl RoomCoordinator for LoggingRoomCoordinator {
    fn mode(&self) -> &'static str {
        "logging"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            "room coordinator activated room lifecycle"
        );
        Ok(())
    }

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            "room coordinator deactivated room lifecycle"
        );
        Ok(())
    }
}

pub struct FileRoomCoordinator {
    node_id: Arc<str>,
    base_url: Option<String>,
    root: PathBuf,
    heartbeat_interval: Duration,
    lease_ttl: Duration,
    heartbeats: Mutex<HashMap<Uuid, RoomLeaseHeartbeat>>,
}

struct RoomLeaseHeartbeat {
    lease_id: Uuid,
    epoch: u64,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

pub struct SqliteRoomCoordinator {
    node_id: Arc<str>,
    base_url: Option<String>,
    path: PathBuf,
    heartbeat_interval: Duration,
    lease_ttl: Duration,
    heartbeats: Mutex<HashMap<Uuid, RoomLeaseHeartbeat>>,
}

pub struct ManagedRoomCoordinator {
    node_id: Arc<str>,
    base_url: Option<String>,
    client: ManagedCoordinationClient,
    heartbeat_interval: Duration,
    lease_ttl: Duration,
    heartbeats: Mutex<HashMap<Uuid, RoomLeaseHeartbeat>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedRoomCoordinatorState {
    pub(crate) doc_id: Uuid,
    pub(crate) node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) lease_id: Option<Uuid>,
    #[serde(default)]
    pub(crate) epoch: u64,
    pub(crate) activated_at: DateTime<Utc>,
    #[serde(default, alias = "updated_at", skip_serializing_if = "Option::is_none")]
    pub(crate) renewed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) expires_at: Option<DateTime<Utc>>,
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

fn parse_room_coordinator_timestamp(
    value: &str,
    doc_id: Uuid,
) -> Result<DateTime<Utc>, RoomCoordinatorError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| {
            RoomCoordinatorError::Operation(format!(
                "persisted coordinator state for document `{doc_id}` contains an invalid RFC3339 timestamp"
            ))
        })
}

fn sqlite_room_coordinator_state_from_row(
    row: SqliteRoomCoordinatorRow,
) -> Result<PersistedRoomCoordinatorState, RoomCoordinatorError> {
    let doc_id = Uuid::parse_str(&row.doc_id).map_err(|_| {
        RoomCoordinatorError::Operation(format!(
            "persisted sqlite room lease row contains an invalid doc_id `{}`",
            row.doc_id
        ))
    })?;
    let lease_id = Uuid::parse_str(&row.lease_id).map_err(|_| {
        RoomCoordinatorError::Operation(format!(
            "persisted sqlite room lease row for document `{doc_id}` contains an invalid lease_id"
        ))
    })?;
    let epoch = u64::try_from(row.epoch).map_err(|_| {
        RoomCoordinatorError::Operation(format!(
            "persisted sqlite room lease row for document `{doc_id}` contains a negative epoch"
        ))
    })?;
    let activated_at = parse_room_coordinator_timestamp(&row.activated_at, doc_id)?;
    let renewed_at = parse_room_coordinator_timestamp(&row.renewed_at, doc_id)?;
    let expires_at = parse_room_coordinator_timestamp(&row.expires_at, doc_id)?;

    Ok(PersistedRoomCoordinatorState {
        doc_id,
        node_id: row.node_id,
        base_url: row.base_url,
        lease_id: Some(lease_id),
        epoch,
        activated_at,
        renewed_at: Some(renewed_at),
        expires_at: Some(expires_at),
    })
}

impl FileRoomCoordinator {
    pub fn new(
        node_id: impl Into<String>,
        base_url: Option<String>,
        root: impl Into<PathBuf>,
        heartbeat_interval: Duration,
        lease_ttl: Duration,
    ) -> Result<Self, RoomCoordinatorError> {
        let node_id = node_id.into();
        let node_id = node_id.trim();
        if node_id.is_empty() {
            return Err(RoomCoordinatorError::Operation(
                "NODE_ID cannot be empty when ROOM_COORDINATOR=file".to_owned(),
            ));
        }

        if heartbeat_interval.is_zero() {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS must be greater than zero when ROOM_COORDINATOR=file".to_owned(),
            ));
        }

        if lease_ttl.is_zero() {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_LEASE_TTL_SECS must be greater than zero when ROOM_COORDINATOR=file".to_owned(),
            ));
        }

        if heartbeat_interval >= lease_ttl {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS must be smaller than ROOM_COORDINATOR_LEASE_TTL_SECS when ROOM_COORDINATOR=file".to_owned(),
            ));
        }

        let base_url = base_url
            .as_deref()
            .map(|value| normalize_origin_url(value, "NODE_BASE_URL"))
            .transpose()
            .map_err(RoomCoordinatorError::Operation)?;

        let root = root.into();
        fs::create_dir_all(&root).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to create room coordinator state dir `{}`: {error}",
                root.display()
            ))
        })?;

        let coordinator = Self {
            node_id: Arc::<str>::from(node_id.to_owned()),
            base_url,
            root,
            heartbeat_interval,
            lease_ttl,
            heartbeats: Mutex::new(HashMap::new()),
        };

        let cleaned_temp_files = coordinator.cleanup_stale_temp_files()?;
        if cleaned_temp_files > 0 {
            info!(
                coordinator_mode = coordinator.mode(),
                node_id = coordinator.node_id(),
                root = %coordinator.root.display(),
                cleaned_temp_files,
                "removed stale temp room coordinator state files during initialization"
            );
        }

        Ok(coordinator)
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    pub fn lease_ttl(&self) -> Duration {
        self.lease_ttl
    }

    fn state_path(&self, doc_id: &Uuid) -> PathBuf {
        self.root.join(format!("{doc_id}.json"))
    }

    fn temp_state_path(&self, doc_id: &Uuid) -> PathBuf {
        self.root
            .join(format!("{doc_id}.json.{}.tmp", Uuid::new_v4()))
    }

    fn temp_state_prefix(&self, doc_id: &Uuid) -> String {
        format!("{doc_id}.json.")
    }

    fn is_temp_state_file_name(file_name: &str) -> bool {
        file_name.ends_with(".tmp") && file_name.contains(".json.")
    }

    fn stale_temp_paths(&self) -> Result<Vec<PathBuf>, RoomCoordinatorError> {
        let mut paths = Vec::new();

        for entry in fs::read_dir(&self.root).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to read room coordinator state dir `{}`: {error}",
                self.root.display()
            ))
        })? {
            let entry = entry.map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to read room coordinator state entry in `{}`: {error}",
                    self.root.display()
                ))
            })?;
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            if Self::is_temp_state_file_name(file_name) {
                paths.push(path);
            }
        }

        Ok(paths)
    }

    fn matching_temp_paths(&self, doc_id: &Uuid) -> Result<Vec<PathBuf>, RoomCoordinatorError> {
        let temp_prefix = self.temp_state_prefix(doc_id);
        let mut paths = Vec::new();

        for path in self.stale_temp_paths()? {
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            if file_name.starts_with(&temp_prefix) {
                paths.push(path);
            }
        }

        Ok(paths)
    }

    fn cleanup_stale_temp_files(&self) -> Result<usize, RoomCoordinatorError> {
        let mut removed = 0;

        for path in self.stale_temp_paths()? {
            match self.remove_file_if_exists(&path) {
                Ok(()) => removed += 1,
                Err(error) => warn!(
                    coordinator_mode = self.mode(),
                    node_id = self.node_id(),
                    path = %path.display(),
                    %error,
                    "failed to remove stale temp room coordinator state file during initialization"
                ),
            }
        }

        Ok(removed)
    }

    fn remove_file_if_exists(&self, path: &Path) -> Result<(), RoomCoordinatorError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(RoomCoordinatorError::Operation(format!(
                "failed to remove room coordinator state `{}`: {error}",
                path.display()
            ))),
        }
    }

    fn write_state_atomically(
        &self,
        doc_id: &Uuid,
        path: &Path,
        bytes: &[u8],
    ) -> Result<(), RoomCoordinatorError> {
        let temp_path = self.temp_state_path(doc_id);

        if let Err(error) = fs::write(&temp_path, bytes) {
            let _ = fs::remove_file(&temp_path);
            return Err(RoomCoordinatorError::Operation(format!(
                "failed to write temp room coordinator state `{}`: {error}",
                temp_path.display()
            )));
        }

        if let Err(error) = fs::rename(&temp_path, path) {
            let _ = fs::remove_file(&temp_path);
            return Err(RoomCoordinatorError::Operation(format!(
                "failed to move room coordinator state into `{}`: {error}",
                path.display()
            )));
        }

        Ok(())
    }

    fn lease_ttl_delta(&self) -> Result<TimeDelta, RoomCoordinatorError> {
        TimeDelta::from_std(self.lease_ttl).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to convert lease TTL to chrono duration: {error}"
            ))
        })
    }

    fn read_state(
        &self,
        doc_id: &Uuid,
    ) -> Result<Option<PersistedRoomCoordinatorState>, RoomCoordinatorError> {
        let path = self.state_path(doc_id);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(RoomCoordinatorError::Operation(format!(
                    "failed to read room coordinator state `{}`: {error}",
                    path.display()
                )));
            }
        };

        let state = serde_json::from_slice(&bytes).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to deserialize room coordinator state `{}`: {error}",
                path.display()
            ))
        })?;

        Ok(Some(state))
    }

    fn write_state(
        &self,
        doc_id: &Uuid,
        state: &PersistedRoomCoordinatorState,
    ) -> Result<(), RoomCoordinatorError> {
        let path = self.state_path(doc_id);
        let bytes = serde_json::to_vec(state).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to serialize room coordinator state `{}`: {error}",
                path.display()
            ))
        })?;

        self.write_state_atomically(doc_id, &path, &bytes)
    }

    fn acquire_lease(
        &self,
        doc_id: &Uuid,
    ) -> Result<PersistedRoomCoordinatorState, RoomCoordinatorError> {
        let now = Utc::now();
        let existing = self.read_state(doc_id)?;

        if let Some(existing_state) = existing.as_ref() {
            if existing_state.doc_id != *doc_id {
                return Err(RoomCoordinatorError::Operation(format!(
                    "persisted coordinator state doc_id `{}` did not match requested doc_id `{doc_id}`",
                    existing_state.doc_id
                )));
            }

            let owner_node_id = existing_state.node_id.trim();
            if owner_node_id.is_empty() {
                return Err(RoomCoordinatorError::Operation(format!(
                    "persisted coordinator state `{}` has an empty node_id",
                    self.state_path(doc_id).display()
                )));
            }

            if owner_node_id != self.node_id() {
                let is_active = existing_state
                    .expires_at
                    .map(|expires_at| expires_at > now)
                    .unwrap_or(true);
                if is_active {
                    return Err(RoomCoordinatorError::Operation(format!(
                        "document `{doc_id}` is already leased by node `{owner_node_id}`"
                    )));
                }
            }
        }

        let lease_id = Uuid::new_v4();
        let epoch = existing
            .as_ref()
            .map(|state| state.epoch.saturating_add(1))
            .unwrap_or(1);
        let expires_at = now + self.lease_ttl_delta()?;
        let state = PersistedRoomCoordinatorState {
            doc_id: *doc_id,
            node_id: self.node_id().to_owned(),
            base_url: self.base_url.clone(),
            lease_id: Some(lease_id),
            epoch,
            activated_at: now,
            renewed_at: Some(now),
            expires_at: Some(expires_at),
        };

        self.write_state(doc_id, &state)?;
        Ok(state)
    }

    fn renew_lease(
        &self,
        doc_id: &Uuid,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<bool, RoomCoordinatorError> {
        let Some(mut state) = self.read_state(doc_id)? else {
            return Ok(false);
        };

        if state.doc_id != *doc_id {
            return Err(RoomCoordinatorError::Operation(format!(
                "persisted coordinator state doc_id `{}` did not match requested doc_id `{doc_id}`",
                state.doc_id
            )));
        }

        if state.node_id.trim() != self.node_id() {
            return Ok(false);
        }

        if state.lease_id != Some(lease_id) || state.epoch != epoch {
            return Ok(false);
        }

        let now = Utc::now();
        state.renewed_at = Some(now);
        state.expires_at = Some(now + self.lease_ttl_delta()?);
        self.write_state(doc_id, &state)?;
        Ok(true)
    }

    fn spawn_heartbeat(
        &self,
        doc_id: Uuid,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<RoomLeaseHeartbeat, RoomCoordinatorError> {
        let coordinator = Self {
            node_id: Arc::clone(&self.node_id),
            base_url: self.base_url.clone(),
            root: self.root.clone(),
            heartbeat_interval: self.heartbeat_interval,
            lease_ttl: self.lease_ttl,
            heartbeats: Mutex::new(HashMap::new()),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop_signal = Arc::clone(&stop);

        let thread = thread::Builder::new()
            .name(format!("room-lease-heartbeat-{doc_id}"))
            .spawn(move || {
                while !stop_signal.load(Ordering::Relaxed) {
                    thread::sleep(coordinator.heartbeat_interval);

                    if stop_signal.load(Ordering::Relaxed) {
                        break;
                    }

                    match coordinator.renew_lease(&doc_id, lease_id, epoch) {
                        Ok(true) => {}
                        Ok(false) => {
                            warn!(
                                coordinator_mode = coordinator.mode(),
                                node_id = coordinator.node_id(),
                                %doc_id,
                                lease_id = %lease_id,
                                epoch,
                                "stopped room lease heartbeat because the active lease changed"
                            );
                            break;
                        }
                        Err(error) => warn!(
                            coordinator_mode = coordinator.mode(),
                            node_id = coordinator.node_id(),
                            %doc_id,
                            lease_id = %lease_id,
                            epoch,
                            %error,
                            "failed to renew file-backed room lease heartbeat"
                        ),
                    }
                }
            })
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to spawn room lease heartbeat thread for `{doc_id}`: {error}"
                ))
            })?;

        Ok(RoomLeaseHeartbeat {
            lease_id,
            epoch,
            stop,
            thread: Some(thread),
        })
    }
}

impl SqliteRoomCoordinator {
    pub fn new(
        node_id: impl Into<String>,
        base_url: Option<String>,
        path: impl Into<PathBuf>,
        heartbeat_interval: Duration,
        lease_ttl: Duration,
    ) -> Result<Self, RoomCoordinatorError> {
        let node_id = node_id.into();
        let node_id = node_id.trim();
        if node_id.is_empty() {
            return Err(RoomCoordinatorError::Operation(
                "NODE_ID cannot be empty when ROOM_COORDINATOR=sqlite".to_owned(),
            ));
        }

        if heartbeat_interval.is_zero() {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS must be greater than zero when ROOM_COORDINATOR=sqlite".to_owned(),
            ));
        }

        if lease_ttl.is_zero() {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_LEASE_TTL_SECS must be greater than zero when ROOM_COORDINATOR=sqlite".to_owned(),
            ));
        }

        if heartbeat_interval >= lease_ttl {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS must be smaller than ROOM_COORDINATOR_LEASE_TTL_SECS when ROOM_COORDINATOR=sqlite".to_owned(),
            ));
        }

        let base_url = base_url
            .as_deref()
            .map(|value| normalize_origin_url(value, "NODE_BASE_URL"))
            .transpose()
            .map_err(RoomCoordinatorError::Operation)?;

        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_SQLITE_PATH cannot be empty when ROOM_COORDINATOR=sqlite"
                    .to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to create sqlite room coordinator parent dir `{}`: {error}",
                    parent.display()
                ))
            })?;
        }

        let coordinator = Self {
            node_id: Arc::<str>::from(node_id.to_owned()),
            base_url,
            path,
            heartbeat_interval,
            lease_ttl,
            heartbeats: Mutex::new(HashMap::new()),
        };
        let connection = coordinator.open_connection()?;
        coordinator.initialize_schema(&connection)?;

        Ok(coordinator)
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    pub fn lease_ttl(&self) -> Duration {
        self.lease_ttl
    }

    fn open_connection(&self) -> Result<Connection, RoomCoordinatorError> {
        let connection = Connection::open(&self.path).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to open sqlite room coordinator database `{}`: {error}",
                self.path.display()
            ))
        })?;
        connection
            .busy_timeout(Duration::from_secs(
                SQLITE_ROOM_COORDINATOR_BUSY_TIMEOUT_SECS,
            ))
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to configure sqlite room coordinator busy timeout `{}`: {error}",
                    self.path.display()
                ))
            })?;
        Ok(connection)
    }

    fn initialize_schema(&self, connection: &Connection) -> Result<(), RoomCoordinatorError> {
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
                RoomCoordinatorError::Operation(format!(
                    "failed to initialize sqlite room coordinator schema `{}`: {error}",
                    self.path.display()
                ))
            })?;
        Ok(())
    }

    fn load_state_from_connection(
        &self,
        connection: &Connection,
        doc_id: &Uuid,
    ) -> Result<Option<PersistedRoomCoordinatorState>, RoomCoordinatorError> {
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
                RoomCoordinatorError::Operation(format!(
                    "failed to read sqlite room lease `{}` for document `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;

        row.map(sqlite_room_coordinator_state_from_row).transpose()
    }

    #[cfg(test)]
    fn load_state(
        &self,
        doc_id: &Uuid,
    ) -> Result<Option<PersistedRoomCoordinatorState>, RoomCoordinatorError> {
        let connection = self.open_connection()?;
        self.initialize_schema(&connection)?;
        self.load_state_from_connection(&connection, doc_id)
    }

    fn lease_ttl_delta(&self) -> Result<TimeDelta, RoomCoordinatorError> {
        TimeDelta::from_std(self.lease_ttl).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to convert lease TTL to chrono duration: {error}"
            ))
        })
    }

    fn acquire_lease(
        &self,
        doc_id: &Uuid,
    ) -> Result<PersistedRoomCoordinatorState, RoomCoordinatorError> {
        let mut connection = self.open_connection()?;
        self.initialize_schema(&connection)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to start sqlite lease acquire transaction `{}`: {error}",
                    self.path.display()
                ))
            })?;

        let now = Utc::now();
        let existing = self.load_state_from_connection(&transaction, doc_id)?;

        if let Some(existing_state) = existing.as_ref() {
            let owner_node_id = existing_state.node_id.trim();
            if owner_node_id.is_empty() {
                return Err(RoomCoordinatorError::Operation(format!(
                    "persisted sqlite coordinator lease for document `{doc_id}` has an empty node_id"
                )));
            }

            if owner_node_id != self.node_id() {
                let is_active = existing_state
                    .expires_at
                    .map(|expires_at| expires_at > now)
                    .unwrap_or(true);
                if is_active {
                    return Err(RoomCoordinatorError::Operation(format!(
                        "document `{doc_id}` is already leased by node `{owner_node_id}`"
                    )));
                }
            }
        }

        let lease_id = Uuid::new_v4();
        let epoch = existing
            .as_ref()
            .map(|state| state.epoch.saturating_add(1))
            .unwrap_or(1);
        let expires_at = now + self.lease_ttl_delta()?;
        let state = PersistedRoomCoordinatorState {
            doc_id: *doc_id,
            node_id: self.node_id().to_owned(),
            base_url: self.base_url.clone(),
            lease_id: Some(lease_id),
            epoch,
            activated_at: now,
            renewed_at: Some(now),
            expires_at: Some(expires_at),
        };

        transaction
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
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(doc_id) DO UPDATE SET
                    node_id = excluded.node_id,
                    base_url = excluded.base_url,
                    lease_id = excluded.lease_id,
                    epoch = excluded.epoch,
                    activated_at = excluded.activated_at,
                    renewed_at = excluded.renewed_at,
                    expires_at = excluded.expires_at",
                params![
                    state.doc_id.to_string(),
                    state.node_id.clone(),
                    state.base_url.clone(),
                    lease_id.to_string(),
                    i64::try_from(state.epoch).map_err(|_| {
                        RoomCoordinatorError::Operation(format!(
                            "lease epoch for document `{doc_id}` exceeded sqlite INTEGER range"
                        ))
                    })?,
                    state.activated_at.to_rfc3339(),
                    state
                        .renewed_at
                        .expect("newly acquired lease should include renewed_at")
                        .to_rfc3339(),
                    state
                        .expires_at
                        .expect("newly acquired lease should include expires_at")
                        .to_rfc3339(),
                ],
            )
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to persist sqlite room lease `{}` for document `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;

        transaction.commit().map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to commit sqlite lease acquire transaction `{}`: {error}",
                self.path.display()
            ))
        })?;

        Ok(state)
    }

    fn renew_lease(
        &self,
        doc_id: &Uuid,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<bool, RoomCoordinatorError> {
        let mut connection = self.open_connection()?;
        self.initialize_schema(&connection)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to start sqlite lease renew transaction `{}`: {error}",
                    self.path.display()
                ))
            })?;

        let Some(mut state) = self.load_state_from_connection(&transaction, doc_id)? else {
            return Ok(false);
        };

        if state.node_id.trim() != self.node_id() {
            return Ok(false);
        }

        if state.lease_id != Some(lease_id) || state.epoch != epoch {
            return Ok(false);
        }

        let now = Utc::now();
        state.renewed_at = Some(now);
        state.expires_at = Some(now + self.lease_ttl_delta()?);

        transaction
            .execute(
                "UPDATE room_leases
                 SET base_url = ?2,
                     renewed_at = ?3,
                     expires_at = ?4
                 WHERE doc_id = ?1
                   AND node_id = ?5
                   AND lease_id = ?6
                   AND epoch = ?7",
                params![
                    doc_id.to_string(),
                    state.base_url.clone(),
                    state
                        .renewed_at
                        .expect("renewed lease should include renewed_at")
                        .to_rfc3339(),
                    state
                        .expires_at
                        .expect("renewed lease should include expires_at")
                        .to_rfc3339(),
                    self.node_id(),
                    lease_id.to_string(),
                    i64::try_from(epoch).map_err(|_| {
                        RoomCoordinatorError::Operation(format!(
                            "lease epoch for document `{doc_id}` exceeded sqlite INTEGER range"
                        ))
                    })?,
                ],
            )
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to renew sqlite room lease `{}` for document `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;

        let renewed = transaction.changes() > 0;
        transaction.commit().map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to commit sqlite lease renew transaction `{}`: {error}",
                self.path.display()
            ))
        })?;

        Ok(renewed)
    }

    fn release_lease(
        &self,
        doc_id: &Uuid,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<bool, RoomCoordinatorError> {
        let mut connection = self.open_connection()?;
        self.initialize_schema(&connection)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to start sqlite lease release transaction `{}`: {error}",
                    self.path.display()
                ))
            })?;

        transaction
            .execute(
                "DELETE FROM room_leases
                 WHERE doc_id = ?1
                   AND node_id = ?2
                   AND lease_id = ?3
                   AND epoch = ?4",
                params![
                    doc_id.to_string(),
                    self.node_id(),
                    lease_id.to_string(),
                    i64::try_from(epoch).map_err(|_| {
                        RoomCoordinatorError::Operation(format!(
                            "lease epoch for document `{doc_id}` exceeded sqlite INTEGER range"
                        ))
                    })?,
                ],
            )
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to release sqlite room lease `{}` for document `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;

        let released = transaction.changes() > 0;
        transaction.commit().map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to commit sqlite lease release transaction `{}`: {error}",
                self.path.display()
            ))
        })?;

        Ok(released)
    }

    fn spawn_heartbeat(
        &self,
        doc_id: Uuid,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<RoomLeaseHeartbeat, RoomCoordinatorError> {
        let coordinator = Self {
            node_id: Arc::clone(&self.node_id),
            base_url: self.base_url.clone(),
            path: self.path.clone(),
            heartbeat_interval: self.heartbeat_interval,
            lease_ttl: self.lease_ttl,
            heartbeats: Mutex::new(HashMap::new()),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop_signal = Arc::clone(&stop);

        let thread = thread::Builder::new()
            .name(format!("sqlite-room-lease-heartbeat-{doc_id}"))
            .spawn(move || {
                while !stop_signal.load(Ordering::Relaxed) {
                    thread::sleep(coordinator.heartbeat_interval);

                    if stop_signal.load(Ordering::Relaxed) {
                        break;
                    }

                    match coordinator.renew_lease(&doc_id, lease_id, epoch) {
                        Ok(true) => {}
                        Ok(false) => {
                            warn!(
                                coordinator_mode = coordinator.mode(),
                                node_id = coordinator.node_id(),
                                %doc_id,
                                lease_id = %lease_id,
                                epoch,
                                "stopped sqlite room lease heartbeat because the active lease changed"
                            );
                            break;
                        }
                        Err(error) => warn!(
                            coordinator_mode = coordinator.mode(),
                            node_id = coordinator.node_id(),
                            %doc_id,
                            lease_id = %lease_id,
                            epoch,
                            %error,
                            "failed to renew sqlite room lease heartbeat"
                        ),
                    }
                }
            })
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to spawn sqlite room lease heartbeat thread for `{doc_id}`: {error}"
                ))
            })?;

        Ok(RoomLeaseHeartbeat {
            lease_id,
            epoch,
            stop,
            thread: Some(thread),
        })
    }
}

impl ManagedRoomCoordinator {
    pub fn new(
        node_id: impl Into<String>,
        base_url: Option<String>,
        managed_base_url: impl Into<String>,
        managed_auth_token: Option<String>,
        managed_timeout: Duration,
        heartbeat_interval: Duration,
        lease_ttl: Duration,
    ) -> Result<Self, RoomCoordinatorError> {
        let node_id = node_id.into();
        let node_id = node_id.trim();
        if node_id.is_empty() {
            return Err(RoomCoordinatorError::Operation(
                "NODE_ID cannot be empty when ROOM_COORDINATOR=managed".to_owned(),
            ));
        }

        if heartbeat_interval.is_zero() {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS must be greater than zero when ROOM_COORDINATOR=managed".to_owned(),
            ));
        }

        if lease_ttl.is_zero() {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_LEASE_TTL_SECS must be greater than zero when ROOM_COORDINATOR=managed".to_owned(),
            ));
        }

        if heartbeat_interval >= lease_ttl {
            return Err(RoomCoordinatorError::Operation(
                "ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS must be smaller than ROOM_COORDINATOR_LEASE_TTL_SECS when ROOM_COORDINATOR=managed".to_owned(),
            ));
        }

        let base_url = base_url
            .as_deref()
            .map(|value| normalize_origin_url(value, "NODE_BASE_URL"))
            .transpose()
            .map_err(RoomCoordinatorError::Operation)?;

        let client =
            ManagedCoordinationClient::new(managed_base_url, managed_auth_token, managed_timeout)
                .map_err(|error| {
                match error {
            ManagedCoordinationClientError::Config(message)
            | ManagedCoordinationClientError::Request(message) => {
                RoomCoordinatorError::Operation(message)
            }
            ManagedCoordinationClientError::Conflict(_) => RoomCoordinatorError::Operation(
                "managed coordination client returned an unexpected conflict during initialization"
                    .to_owned(),
            ),
        }
            })?;

        Ok(Self {
            node_id: Arc::<str>::from(node_id.to_owned()),
            base_url,
            client,
            heartbeat_interval,
            lease_ttl,
            heartbeats: Mutex::new(HashMap::new()),
        })
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    pub fn lease_ttl(&self) -> Duration {
        self.lease_ttl
    }

    fn acquire_lease(
        &self,
        doc_id: &Uuid,
    ) -> Result<PersistedRoomCoordinatorState, RoomCoordinatorError> {
        self.client
            .acquire_lease(
                doc_id,
                self.node_id(),
                self.base_url.as_deref(),
                self.lease_ttl,
            )
            .map_err(|error| match error {
                ManagedCoordinationClientError::Conflict(Some(state)) => {
                    let owner_node_id = state.node_id.trim();
                    let owner_node_id = if owner_node_id.is_empty() {
                        "<unknown-node>"
                    } else {
                        owner_node_id
                    };
                    RoomCoordinatorError::Operation(format!(
                        "document `{doc_id}` is already leased by node `{owner_node_id}`"
                    ))
                }
                ManagedCoordinationClientError::Conflict(None) => RoomCoordinatorError::Operation(
                    format!("document `{doc_id}` is already leased by another collaboration node"),
                ),
                ManagedCoordinationClientError::Config(message)
                | ManagedCoordinationClientError::Request(message) => {
                    RoomCoordinatorError::Operation(message)
                }
            })
    }

    fn renew_lease(
        &self,
        doc_id: &Uuid,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<bool, RoomCoordinatorError> {
        self.client
            .renew_lease(doc_id, self.node_id(), lease_id, epoch, self.lease_ttl)
            .map(|state| state.is_some())
            .map_err(|error| match error {
                ManagedCoordinationClientError::Config(message)
                | ManagedCoordinationClientError::Request(message) => {
                    RoomCoordinatorError::Operation(message)
                }
                ManagedCoordinationClientError::Conflict(_) => RoomCoordinatorError::Operation(
                    format!(
                        "managed coordination renew unexpectedly returned a conflict for document `{doc_id}`"
                    ),
                ),
            })
    }

    fn release_lease(
        &self,
        doc_id: &Uuid,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<bool, RoomCoordinatorError> {
        self.client
            .release_lease(doc_id, self.node_id(), lease_id, epoch)
            .map_err(|error| match error {
                ManagedCoordinationClientError::Config(message)
                | ManagedCoordinationClientError::Request(message) => {
                    RoomCoordinatorError::Operation(message)
                }
                ManagedCoordinationClientError::Conflict(_) => RoomCoordinatorError::Operation(
                    format!(
                        "managed coordination release unexpectedly returned a conflict for document `{doc_id}`"
                    ),
                ),
            })
    }

    fn spawn_heartbeat(
        &self,
        doc_id: Uuid,
        lease_id: Uuid,
        epoch: u64,
    ) -> Result<RoomLeaseHeartbeat, RoomCoordinatorError> {
        let coordinator = Self {
            node_id: Arc::clone(&self.node_id),
            base_url: self.base_url.clone(),
            client: self.client.clone(),
            heartbeat_interval: self.heartbeat_interval,
            lease_ttl: self.lease_ttl,
            heartbeats: Mutex::new(HashMap::new()),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop_signal = Arc::clone(&stop);

        let thread = thread::Builder::new()
            .name(format!("managed-room-lease-heartbeat-{doc_id}"))
            .spawn(move || {
                while !stop_signal.load(Ordering::Relaxed) {
                    thread::sleep(coordinator.heartbeat_interval);

                    if stop_signal.load(Ordering::Relaxed) {
                        break;
                    }

                    match coordinator.renew_lease(&doc_id, lease_id, epoch) {
                        Ok(true) => {}
                        Ok(false) => {
                            warn!(
                                coordinator_mode = coordinator.mode(),
                                node_id = coordinator.node_id(),
                                %doc_id,
                                lease_id = %lease_id,
                                epoch,
                                "stopped managed room lease heartbeat because the active lease changed"
                            );
                            break;
                        }
                        Err(error) => warn!(
                            coordinator_mode = coordinator.mode(),
                            node_id = coordinator.node_id(),
                            %doc_id,
                            lease_id = %lease_id,
                            epoch,
                            %error,
                            "failed to renew managed room lease heartbeat"
                        ),
                    }
                }
            })
            .map_err(|error| {
                RoomCoordinatorError::Operation(format!(
                    "failed to spawn managed room lease heartbeat thread for `{doc_id}`: {error}"
                ))
            })?;

        Ok(RoomLeaseHeartbeat {
            lease_id,
            epoch,
            stop,
            thread: Some(thread),
        })
    }
}

impl RoomCoordinator for FileRoomCoordinator {
    fn mode(&self) -> &'static str {
        "file"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        let state = self.acquire_lease(doc_id)?;
        let lease_id = state
            .lease_id
            .expect("newly acquired lease should always have an id");
        let heartbeat = self.spawn_heartbeat(*doc_id, lease_id, state.epoch)?;

        let mut heartbeats = self
            .heartbeats
            .lock()
            .expect("file room coordinator heartbeat registry should not be poisoned");
        if heartbeats.contains_key(doc_id) {
            heartbeat.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = heartbeat.thread {
                let _ = thread.join();
            }
            return Err(RoomCoordinatorError::Operation(format!(
                "document `{doc_id}` already has an active lease heartbeat on this node"
            )));
        }
        heartbeats.insert(*doc_id, heartbeat);

        let path = self.state_path(doc_id);
        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            lease_id = %lease_id,
            epoch = state.epoch,
            expires_at = %state
                .expires_at
                .expect("newly acquired lease should always have an expiry"),
            path = %path.display(),
            "persisted file-backed room lease state"
        );
        Ok(())
    }

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        let heartbeat = self
            .heartbeats
            .lock()
            .expect("file room coordinator heartbeat registry should not be poisoned")
            .remove(doc_id);

        let Some(mut heartbeat) = heartbeat else {
            return Err(RoomCoordinatorError::Operation(format!(
                "document `{doc_id}` does not have an active lease heartbeat on this node"
            )));
        };

        heartbeat.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = heartbeat.thread.take() {
            let _ = thread.join();
        }

        let path = self.state_path(doc_id);
        let state = self.read_state(doc_id)?;
        let should_release = state
            .as_ref()
            .map(|state| {
                state.node_id.trim() == self.node_id()
                    && state.lease_id == Some(heartbeat.lease_id)
                    && state.epoch == heartbeat.epoch
            })
            .unwrap_or(false);

        if should_release {
            self.remove_file_if_exists(&path)?;
        }

        for temp_path in self.matching_temp_paths(doc_id)? {
            self.remove_file_if_exists(&temp_path)?;
        }

        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            lease_id = %heartbeat.lease_id,
            epoch = heartbeat.epoch,
            released = should_release,
            path = %path.display(),
            "released file-backed room lease state"
        );
        Ok(())
    }
}

impl Drop for FileRoomCoordinator {
    fn drop(&mut self) {
        let mut heartbeats = self
            .heartbeats
            .lock()
            .expect("file room coordinator heartbeat registry should not be poisoned");

        for (_doc_id, heartbeat) in heartbeats.iter_mut() {
            heartbeat.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = heartbeat.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

impl RoomCoordinator for SqliteRoomCoordinator {
    fn mode(&self) -> &'static str {
        "sqlite"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        let state = self.acquire_lease(doc_id)?;
        let lease_id = state
            .lease_id
            .expect("newly acquired sqlite lease should always have an id");
        let heartbeat = self.spawn_heartbeat(*doc_id, lease_id, state.epoch)?;

        let mut heartbeats = self
            .heartbeats
            .lock()
            .expect("sqlite room coordinator heartbeat registry should not be poisoned");
        if heartbeats.contains_key(doc_id) {
            heartbeat.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = heartbeat.thread {
                let _ = thread.join();
            }
            return Err(RoomCoordinatorError::Operation(format!(
                "document `{doc_id}` already has an active lease heartbeat on this node"
            )));
        }
        heartbeats.insert(*doc_id, heartbeat);

        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            lease_id = %lease_id,
            epoch = state.epoch,
            expires_at = %state
                .expires_at
                .expect("newly acquired sqlite lease should always have an expiry"),
            path = %self.path.display(),
            "persisted sqlite-backed room lease state"
        );
        Ok(())
    }

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        let heartbeat = self
            .heartbeats
            .lock()
            .expect("sqlite room coordinator heartbeat registry should not be poisoned")
            .remove(doc_id);

        let Some(mut heartbeat) = heartbeat else {
            return Err(RoomCoordinatorError::Operation(format!(
                "document `{doc_id}` does not have an active lease heartbeat on this node"
            )));
        };

        heartbeat.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = heartbeat.thread.take() {
            let _ = thread.join();
        }

        let released = self.release_lease(doc_id, heartbeat.lease_id, heartbeat.epoch)?;
        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            lease_id = %heartbeat.lease_id,
            epoch = heartbeat.epoch,
            released,
            path = %self.path.display(),
            "released sqlite-backed room lease state"
        );
        Ok(())
    }
}

impl Drop for SqliteRoomCoordinator {
    fn drop(&mut self) {
        let mut heartbeats = self
            .heartbeats
            .lock()
            .expect("sqlite room coordinator heartbeat registry should not be poisoned");

        for (_doc_id, heartbeat) in heartbeats.iter_mut() {
            heartbeat.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = heartbeat.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

impl RoomCoordinator for ManagedRoomCoordinator {
    fn mode(&self) -> &'static str {
        "managed"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        let state = self.acquire_lease(doc_id)?;
        let lease_id = state
            .lease_id
            .expect("newly acquired managed lease should always have an id");
        let heartbeat = self.spawn_heartbeat(*doc_id, lease_id, state.epoch)?;

        let mut heartbeats = self
            .heartbeats
            .lock()
            .expect("managed room coordinator heartbeat registry should not be poisoned");
        if heartbeats.contains_key(doc_id) {
            heartbeat.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = heartbeat.thread {
                let _ = thread.join();
            }
            return Err(RoomCoordinatorError::Operation(format!(
                "document `{doc_id}` already has an active lease heartbeat on this node"
            )));
        }
        heartbeats.insert(*doc_id, heartbeat);

        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            lease_id = %lease_id,
            epoch = state.epoch,
            expires_at = %state
                .expires_at
                .expect("newly acquired managed lease should always have an expiry"),
            "persisted managed room lease state"
        );
        Ok(())
    }

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        let heartbeat = self
            .heartbeats
            .lock()
            .expect("managed room coordinator heartbeat registry should not be poisoned")
            .remove(doc_id);

        let Some(mut heartbeat) = heartbeat else {
            return Err(RoomCoordinatorError::Operation(format!(
                "document `{doc_id}` does not have an active lease heartbeat on this node"
            )));
        };

        heartbeat.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = heartbeat.thread.take() {
            let _ = thread.join();
        }

        let released = self.release_lease(doc_id, heartbeat.lease_id, heartbeat.epoch)?;
        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            lease_id = %heartbeat.lease_id,
            epoch = heartbeat.epoch,
            released,
            "released managed room lease state"
        );
        Ok(())
    }
}

impl Drop for ManagedRoomCoordinator {
    fn drop(&mut self) {
        let mut heartbeats = self
            .heartbeats
            .lock()
            .expect("managed room coordinator heartbeat registry should not be poisoned");

        for (_doc_id, heartbeat) in heartbeats.iter_mut() {
            heartbeat.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = heartbeat.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

pub fn noop_room_coordinator() -> Arc<dyn RoomCoordinator> {
    Arc::new(NoopRoomCoordinator)
}

pub fn logging_room_coordinator(node_id: impl Into<String>) -> Arc<dyn RoomCoordinator> {
    Arc::new(LoggingRoomCoordinator::new(node_id))
}

pub fn file_room_coordinator(
    node_id: impl Into<String>,
    base_url: Option<String>,
    root: impl Into<PathBuf>,
    heartbeat_interval: Duration,
    lease_ttl: Duration,
) -> Result<Arc<dyn RoomCoordinator>, RoomCoordinatorError> {
    Ok(Arc::new(FileRoomCoordinator::new(
        node_id,
        base_url,
        root,
        heartbeat_interval,
        lease_ttl,
    )?))
}

pub fn sqlite_room_coordinator(
    node_id: impl Into<String>,
    base_url: Option<String>,
    path: impl Into<PathBuf>,
    heartbeat_interval: Duration,
    lease_ttl: Duration,
) -> Result<Arc<dyn RoomCoordinator>, RoomCoordinatorError> {
    Ok(Arc::new(SqliteRoomCoordinator::new(
        node_id,
        base_url,
        path,
        heartbeat_interval,
        lease_ttl,
    )?))
}

pub fn managed_room_coordinator(
    node_id: impl Into<String>,
    base_url: Option<String>,
    managed_base_url: impl Into<String>,
    managed_auth_token: Option<String>,
    managed_timeout: Duration,
    heartbeat_interval: Duration,
    lease_ttl: Duration,
) -> Result<Arc<dyn RoomCoordinator>, RoomCoordinatorError> {
    Ok(Arc::new(ManagedRoomCoordinator::new(
        node_id,
        base_url,
        managed_base_url,
        managed_auth_token,
        managed_timeout,
        heartbeat_interval,
        lease_ttl,
    )?))
}

pub fn room_coordinator_from_config(config: &Config) -> AppResult<Arc<dyn RoomCoordinator>> {
    match config.room_coordinator.trim().to_ascii_lowercase().as_str() {
        "noop" => Ok(noop_room_coordinator()),
        "logging" => Ok(logging_room_coordinator(config.node_id.clone())),
        "file" => file_room_coordinator(
            config.node_id.clone(),
            config.node_base_url.clone(),
            config.room_coordinator_state_dir.clone(),
            Duration::from_secs(config.room_coordinator_heartbeat_interval_secs),
            Duration::from_secs(config.room_coordinator_lease_ttl_secs),
        )
        .map_err(|error| {
            AppError::Config(format!(
                "failed to initialize file room coordinator: {error}"
            ))
        }),
        "sqlite" => sqlite_room_coordinator(
            config.node_id.clone(),
            config.node_base_url.clone(),
            config.room_coordinator_sqlite_path.clone(),
            Duration::from_secs(config.room_coordinator_heartbeat_interval_secs),
            Duration::from_secs(config.room_coordinator_lease_ttl_secs),
        )
        .map_err(|error| {
            AppError::Config(format!(
                "failed to initialize sqlite room coordinator: {error}"
            ))
        }),
        "managed" => {
            let managed_base_url = config
                .room_coordination_managed_base_url
                .clone()
                .ok_or_else(|| {
                    AppError::Config(
                        "ROOM_COORDINATION_MANAGED_BASE_URL is required when ROOM_COORDINATOR=managed"
                            .to_owned(),
                    )
                })?;
            managed_room_coordinator(
                config.node_id.clone(),
                config.node_base_url.clone(),
                managed_base_url,
                config.room_coordination_managed_auth_token.clone(),
                Duration::from_secs(config.room_coordination_managed_timeout_secs),
                Duration::from_secs(config.room_coordinator_heartbeat_interval_secs),
                Duration::from_secs(config.room_coordinator_lease_ttl_secs),
            )
            .map_err(|error| {
                AppError::Config(format!(
                    "failed to initialize managed room coordinator: {error}"
                ))
            })
        }
        other => Err(AppError::Config(format!(
            "ROOM_COORDINATOR must be `noop`, `logging`, `file`, `sqlite`, or `managed`, received `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, thread, time::Duration};

    fn temp_state_dir(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "backend-room-coordinator-{test_name}-{}",
            Uuid::new_v4()
        ))
    }

    fn temp_sqlite_path(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "backend-room-coordinator-{test_name}-{}.sqlite3",
            Uuid::new_v4()
        ))
    }

    fn test_config(room_coordinator: &str) -> Config {
        Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_deeb_path: "./data/test-snapshots.deeb.json".to_owned(),
            snapshot_agdb_path: "./data/test-snapshots.agdb".to_owned(),
            snapshot_amandine_path: "./data/test-snapshots.amandine".to_owned(),
            snapshot_apex_store_path: "./data/test-snapshots.apex_store".to_owned(),
            snapshot_armdb_path: "./data/test-snapshots.armdb".to_owned(),
            snapshot_assystem_path: "./data/test-snapshots.assystem".to_owned(),
            snapshot_colon_db_path: "./data/test-snapshots.colon_db".to_owned(),
            snapshot_dharmadb_path: "./data/test-snapshots.dharmadb".to_owned(),
            snapshot_dir_cache_path: "./data/test-snapshots.dir_cache".to_owned(),
            snapshot_flash_kv_path: "./data/test-snapshots.flash_kv".to_owned(),
            snapshot_ghaladb_path: "./data/test-snapshots.ghaladb".to_owned(),
            snapshot_blockbucket_path: "./data/test-snapshots.blockbucket".to_owned(),
            snapshot_grebedb_path: "./data/test-snapshots.grebedb".to_owned(),
            snapshot_grumpydb_path: "./data/test-snapshots.grumpydb".to_owned(),
            snapshot_highlandcows_isam_path: "./data/test-snapshots.highlandcows_isam".to_owned(),
            snapshot_simple_db_path: "./data/test-snapshots.simple_db".to_owned(),
            snapshot_docdb_path: "./data/test-snapshots.docdb.json".to_owned(),
            snapshot_eight_path: "./data/test-snapshots.eight".to_owned(),
            snapshot_epoch_db_path: "./data/test-snapshots.epoch_db".to_owned(),
            snapshot_etchdb_path: "./data/test-snapshots.etchdb".to_owned(),
            snapshot_ferrumdb_path: "./data/test-snapshots.ferrumdb".to_owned(),
            snapshot_rumdb_path: "./data/test-snapshots.rumdb".to_owned(),
            snapshot_rubin_path: "./data/test-snapshots.rubin.json".to_owned(),
            snapshot_shorterdb_path: "./data/test-snapshots.shorterdb".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
            snapshot_hightower_kv_path: "./data/test-snapshots.hightower_kv".to_owned(),
            snapshot_hmdb_path: "./data/test-snapshots.hmdb".to_owned(),
            snapshot_icefalldb_path: "./data/test-snapshots.icefalldb".to_owned(),
            snapshot_bitask_path: "./data/test-snapshots.bitask".to_owned(),
            snapshot_bitkv_rs_path: "./data/test-snapshots.bitkv_rs".to_owned(),
            snapshot_bitcask_engine_path: "./data/test-snapshots.bitcask_engine".to_owned(),
            snapshot_blazeup_path: "./data/test-snapshots.blazeup".to_owned(),
            snapshot_candystore_path: "./data/test-snapshots.candystore".to_owned(),
            snapshot_celerix_store_path: "./data/test-snapshots.celerix_store".to_owned(),
            snapshot_citadeldb_path: "./data/test-snapshots.citadeldb".to_owned(),
            snapshot_citadeldb_passphrase: "test-citadel-snapshot-passphrase".to_owned(),
            snapshot_cuendillar_path: "./data/test-snapshots.cuendillar".to_owned(),
            snapshot_datastack_path: "./data/test-snapshots.datastack".to_owned(),
            snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
            snapshot_mace_path: "./data/test-snapshots.mace".to_owned(),
            snapshot_janql_path: "./data/test-snapshots.janql".to_owned(),
            snapshot_jasondb_path: "./data/test-snapshots.jasondb".to_owned(),
            snapshot_jasonisnthappy_path: "./data/test-snapshots.jasonisnthappy".to_owned(),
            snapshot_jfs_path: "./data/test-snapshots.jfs.json".to_owned(),
            snapshot_json_store_path: "./data/test-snapshots.json_store.jsonl".to_owned(),
            snapshot_json_mutex_db_path: "./data/test-snapshots.json_mutex_db.json".to_owned(),
            snapshot_feoxdb_path: "./data/test-snapshots.feoxdb".to_owned(),
            snapshot_jsondb_path: "./data/test-snapshots.jsondb.json".to_owned(),
            snapshot_kopperdb_path: "./data/test-snapshots.kopperdb".to_owned(),
            snapshot_kv_path: "./data/test-snapshots.kv".to_owned(),
            snapshot_koit_path: "./data/test-snapshots.koit.json".to_owned(),
            snapshot_lite_db_path: "./data/test-snapshots.lite_db".to_owned(),
            snapshot_log_kv_path: "./data/test-snapshots.log_kv".to_owned(),
            snapshot_append_kv_path: "./data/test-snapshots.append_kv".to_owned(),
            snapshot_mhdb_path: "./data/test-snapshots.mhdb".to_owned(),
            snapshot_loro_kv_path: "./data/test-snapshots.loro_kv".to_owned(),
            snapshot_luckdb_path: "./data/test-snapshots.luckdb.json".to_owned(),
            snapshot_ipjdb_path: "./data/test-snapshots.ipjdb".to_owned(),
            snapshot_lsm_engine_path: "./data/test-snapshots.lsm_engine".to_owned(),
            snapshot_lsm_storage_engine_path: "./data/test-snapshots.lsm_storage_engine".to_owned(),
            snapshot_lsmdb_path: "./data/test-snapshots.lsmdb".to_owned(),
            snapshot_lsm_tree_path: "./data/test-snapshots.lsm_tree".to_owned(),
            snapshot_mindb_path: "./data/test-snapshots.mindb".to_owned(),
            snapshot_mmdb_path: "./data/test-snapshots.mmdb".to_owned(),
            snapshot_mu_db_path: "./data/test-snapshots.mu_db".to_owned(),
            snapshot_nanodb_path: "./data/test-snapshots.nanodb.json".to_owned(),
            snapshot_graus_db_path: "./data/test-snapshots.graus_db".to_owned(),
            snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
            snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
            snapshot_persistent_kv_path: "./data/test-snapshots.persistent_kv".to_owned(),
            snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
            snapshot_nebari_path: "./data/test-snapshots.nebari".to_owned(),
            snapshot_nikidb_path: "./data/test-snapshots.nikidb".to_owned(),
            snapshot_nodb_path: "./data/test-snapshots.nodb".to_owned(),
            snapshot_okofdb_path: "./data/test-snapshots.okofdb".to_owned(),
            snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
            snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
            snapshot_rskey_path: "./data/test-snapshots.rskey".to_owned(),
            snapshot_readb_path: "./data/test-snapshots.readb".to_owned(),
            snapshot_rustlite_path: "./data/test-snapshots.rustlite".to_owned(),
            snapshot_rustcask_path: "./data/test-snapshots.rustcask".to_owned(),
            snapshot_rusty_leveldb_path: "./data/test-snapshots.rusty_leveldb".to_owned(),
            snapshot_canopydb_path: "./data/test-snapshots.canopydb".to_owned(),
            snapshot_caves_path: "./data/test-snapshots.caves".to_owned(),
            snapshot_ckydb_path: "./data/test-snapshots.ckydb".to_owned(),
            snapshot_crepedb_path: "./data/test-snapshots.crepedb".to_owned(),
            snapshot_crystal_path: "./data/test-snapshots.crystal".to_owned(),
            snapshot_scdb_path: "./data/test-snapshots.scdb".to_owned(),
            snapshot_skv_path: "./data/test-snapshots.skv".to_owned(),
            snapshot_surrealkv_path: "./data/test-snapshots.surrealkv".to_owned(),
            snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
            snapshot_rcask_path: "./data/test-snapshots.rcask".to_owned(),
            snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
            snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
            snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
            snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
            snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
            snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
            snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
            snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
            snapshot_aeternusdb_path: "./data/test-snapshots.aeternusdb".to_owned(),
            snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
            snapshot_thetadb_path: "./data/test-snapshots.thetadb".to_owned(),
            snapshot_tinybase_path: "./data/test-snapshots.tinybase".to_owned(),
            snapshot_tinydb_path: "./data/test-snapshots.tinydb".to_owned(),
            snapshot_dblite_path: "./data/test-snapshots.dblite".to_owned(),
            snapshot_dbless_path: "./data/test-snapshots.dbless".to_owned(),
            snapshot_db_rs_path: "./data/test-snapshots.db_rs".to_owned(),
            snapshot_sanakirja_path: "./data/test-snapshots.sanakirja".to_owned(),
            snapshot_snaildb_path: "./data/test-snapshots.snaildb".to_owned(),
            snapshot_tinykv_path: "./data/test-snapshots.tinykv.json".to_owned(),
            snapshot_vsdb_path: "./data/test-snapshots.vsdb".to_owned(),
            snapshot_yakv_path: "./data/test-snapshots.yakv".to_owned(),
            snapshot_yakvdb_path: "./data/test-snapshots.yakvdb".to_owned(),
            snapshot_saberdb_path: "./data/test-snapshots.saberdb.json".to_owned(),
            snapshot_smolldb_path: "./data/test-snapshots.smolldb".to_owned(),
            snapshot_kstone_path: "./data/test-snapshots.kstone".to_owned(),
            snapshot_roughdb_path: "./data/test-snapshots.roughdb".to_owned(),
            snapshot_raindb_path: "./data/test-snapshots.raindb".to_owned(),
            snapshot_infusedb_path: "./data/test-snapshots.infusedb".to_owned(),
            snapshot_kafi_path: "./data/test-snapshots.kafi".to_owned(),
            snapshot_tinkv_path: "./data/test-snapshots.tinkv".to_owned(),
            snapshot_ledger_kv_path: "./data/test-snapshots.ledger_kv".to_owned(),
            snapshot_joydb_path: "./data/test-snapshots.joydb.json".to_owned(),
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
            room_locator: "local".to_owned(),
            room_coordinator: room_coordinator.to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
            room_coordinator_heartbeat_interval_secs: 1,
            room_coordinator_lease_ttl_secs: 5,
            room_coordination_managed_base_url: None,
            room_coordination_managed_auth_token: None,
            room_coordination_managed_timeout_secs: 5,
            node_id: "node-a".to_owned(),
            node_base_url: None,
            room_owner_hints_path: None,
        }
    }

    #[test]
    fn logging_room_coordinator_uses_trimmed_node_id() {
        let coordinator = LoggingRoomCoordinator::new("  node-a  ");

        assert_eq!(coordinator.mode(), "logging");
        assert_eq!(coordinator.node_id(), "node-a");
        coordinator
            .room_activated(&Uuid::new_v4())
            .expect("logging coordinator should allow room activation");
        coordinator
            .room_deactivated(&Uuid::new_v4())
            .expect("logging coordinator should allow room deactivation");
    }

    #[test]
    fn room_coordinator_from_config_loads_logging_mode() {
        let coordinator = room_coordinator_from_config(&test_config("logging"))
            .expect("config should produce a logging room coordinator");

        assert_eq!(coordinator.mode(), "logging");
    }

    #[test]
    fn file_room_coordinator_persists_and_clears_active_room_state() {
        let root = temp_state_dir("persist-state");
        let coordinator = FileRoomCoordinator::new(
            "node-a",
            Some("http://node-a.internal:4000/".to_owned()),
            &root,
            Duration::from_millis(25),
            Duration::from_millis(80),
        )
        .expect("file room coordinator should initialize");
        let doc_id = Uuid::new_v4();
        let state_path = root.join(format!("{doc_id}.json"));

        coordinator
            .room_activated(&doc_id)
            .expect("file coordinator should persist active room state");

        let state = fs::read_to_string(&state_path)
            .expect("file coordinator should persist room state as json");
        let state: PersistedRoomCoordinatorState =
            serde_json::from_str(&state).expect("room state json should deserialize");
        assert_eq!(state.doc_id, doc_id);
        assert_eq!(state.node_id, "node-a");
        assert!(state.lease_id.is_some());
        assert_eq!(state.epoch, 1);
        assert_eq!(
            state.base_url,
            Some("http://node-a.internal:4000".to_owned())
        );
        let renewed_at = state.renewed_at.expect("lease should include renewed_at");
        assert!(renewed_at >= state.activated_at);
        assert!(state.expires_at.expect("lease should include expiry") > renewed_at);

        coordinator
            .room_deactivated(&doc_id)
            .expect("file coordinator should remove active room state");
        assert!(!state_path.exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn room_coordinator_from_config_loads_file_mode() {
        let root = temp_state_dir("config-file-mode");
        let mut config = test_config("file");
        config.room_coordinator_state_dir = root.display().to_string();

        let coordinator = room_coordinator_from_config(&config)
            .expect("config should produce a file room coordinator");

        assert_eq!(coordinator.mode(), "file");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sqlite_room_coordinator_persists_and_clears_active_room_state() {
        let path = temp_sqlite_path("sqlite-persist-state");
        let coordinator = SqliteRoomCoordinator::new(
            "node-a",
            Some("http://node-a.internal:4000/".to_owned()),
            &path,
            Duration::from_millis(25),
            Duration::from_millis(80),
        )
        .expect("sqlite room coordinator should initialize");
        let doc_id = Uuid::new_v4();

        coordinator
            .room_activated(&doc_id)
            .expect("sqlite coordinator should persist active room state");

        let state = coordinator
            .load_state(&doc_id)
            .expect("sqlite coordinator should load persisted room state")
            .expect("sqlite coordinator should persist a room lease row");
        assert_eq!(state.doc_id, doc_id);
        assert_eq!(state.node_id, "node-a");
        assert!(state.lease_id.is_some());
        assert_eq!(state.epoch, 1);
        assert_eq!(
            state.base_url,
            Some("http://node-a.internal:4000".to_owned())
        );
        let renewed_at = state.renewed_at.expect("lease should include renewed_at");
        assert!(renewed_at >= state.activated_at);
        assert!(state.expires_at.expect("lease should include expiry") > renewed_at);

        coordinator
            .room_deactivated(&doc_id)
            .expect("sqlite coordinator should remove active room state");
        assert!(
            coordinator
                .load_state(&doc_id)
                .expect("sqlite coordinator should query room lease state")
                .is_none()
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn room_coordinator_from_config_loads_sqlite_mode() {
        let path = temp_sqlite_path("config-sqlite-mode");
        let mut config = test_config("sqlite");
        config.room_coordinator_sqlite_path = path.display().to_string();

        let coordinator = room_coordinator_from_config(&config)
            .expect("config should produce a sqlite room coordinator");

        assert_eq!(coordinator.mode(), "sqlite");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn file_room_coordinator_heartbeat_renews_lease_expiry() {
        let root = temp_state_dir("heartbeat-renew");
        let coordinator = FileRoomCoordinator::new(
            "node-a",
            None,
            &root,
            Duration::from_millis(20),
            Duration::from_millis(90),
        )
        .expect("file room coordinator should initialize");
        let doc_id = Uuid::new_v4();
        let state_path = root.join(format!("{doc_id}.json"));

        coordinator
            .room_activated(&doc_id)
            .expect("file coordinator should acquire room lease");

        let initial_state: PersistedRoomCoordinatorState =
            serde_json::from_slice(&fs::read(&state_path).expect("state file should exist"))
                .expect("state file should deserialize");
        let initial_renewed_at = initial_state
            .renewed_at
            .expect("initial state should include renewed_at");
        let initial_expires_at = initial_state
            .expires_at
            .expect("state file should include lease expiry");

        thread::sleep(Duration::from_millis(60));

        let renewed_state: PersistedRoomCoordinatorState =
            serde_json::from_slice(&fs::read(&state_path).expect("state file should exist"))
                .expect("state file should deserialize");
        assert_eq!(renewed_state.lease_id, initial_state.lease_id);
        assert_eq!(renewed_state.epoch, initial_state.epoch);
        assert!(
            renewed_state
                .renewed_at
                .expect("renewed state should include renewed_at")
                > initial_renewed_at
        );
        assert!(
            renewed_state
                .expires_at
                .expect("renewed state should include lease expiry")
                > initial_expires_at
        );

        coordinator
            .room_deactivated(&doc_id)
            .expect("file coordinator should release room lease");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sqlite_room_coordinator_heartbeat_renews_lease_expiry() {
        let path = temp_sqlite_path("sqlite-heartbeat-renew");
        let coordinator = SqliteRoomCoordinator::new(
            "node-a",
            None,
            &path,
            Duration::from_millis(20),
            Duration::from_millis(90),
        )
        .expect("sqlite room coordinator should initialize");
        let doc_id = Uuid::new_v4();

        coordinator
            .room_activated(&doc_id)
            .expect("sqlite coordinator should acquire room lease");

        let initial_state = coordinator
            .load_state(&doc_id)
            .expect("sqlite coordinator should load initial room lease state")
            .expect("sqlite coordinator should persist room lease state");
        let initial_renewed_at = initial_state
            .renewed_at
            .expect("initial state should include renewed_at");
        let initial_expires_at = initial_state
            .expires_at
            .expect("state row should include lease expiry");

        thread::sleep(Duration::from_millis(60));

        let renewed_state = coordinator
            .load_state(&doc_id)
            .expect("sqlite coordinator should load renewed room lease state")
            .expect("sqlite coordinator should keep room lease state");
        assert_eq!(renewed_state.lease_id, initial_state.lease_id);
        assert_eq!(renewed_state.epoch, initial_state.epoch);
        assert!(
            renewed_state
                .renewed_at
                .expect("renewed state should include renewed_at")
                > initial_renewed_at
        );
        assert!(
            renewed_state
                .expires_at
                .expect("renewed state should include lease expiry")
                > initial_expires_at
        );

        coordinator
            .room_deactivated(&doc_id)
            .expect("sqlite coordinator should release room lease");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn file_room_coordinator_rejects_active_lease_from_other_node() {
        let root = temp_state_dir("reject-other-node");
        let coordinator = FileRoomCoordinator::new(
            "node-a",
            None,
            &root,
            Duration::from_secs(1),
            Duration::from_secs(5),
        )
        .expect("file room coordinator should initialize");
        let doc_id = Uuid::new_v4();
        let state_path = root.join(format!("{doc_id}.json"));
        let now = Utc::now();

        fs::write(
            &state_path,
            serde_json::to_vec(&PersistedRoomCoordinatorState {
                doc_id,
                node_id: "node-b".to_owned(),
                base_url: None,
                lease_id: Some(Uuid::new_v4()),
                epoch: 7,
                activated_at: now,
                renewed_at: Some(now),
                expires_at: Some(now + TimeDelta::seconds(30)),
            })
            .expect("state should serialize"),
        )
        .expect("state should be written");

        let error = coordinator
            .room_activated(&doc_id)
            .expect_err("active lease from another node should reject acquisition");
        assert_eq!(
            error.to_string(),
            format!(
                "room coordination failed: document `{doc_id}` is already leased by node `node-b`"
            )
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sqlite_room_coordinator_rejects_active_lease_from_other_node() {
        let path = temp_sqlite_path("sqlite-reject-other-node");
        let coordinator = SqliteRoomCoordinator::new(
            "node-a",
            None,
            &path,
            Duration::from_secs(1),
            Duration::from_secs(5),
        )
        .expect("sqlite room coordinator should initialize");
        let doc_id = Uuid::new_v4();
        let now = Utc::now();
        let connection = Connection::open(&path).expect("sqlite file should be writable");
        coordinator
            .initialize_schema(&connection)
            .expect("sqlite schema should initialize");
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
                params![
                    doc_id.to_string(),
                    "node-b",
                    Uuid::new_v4().to_string(),
                    7_i64,
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                    (now + TimeDelta::seconds(30)).to_rfc3339(),
                ],
            )
            .expect("active sqlite lease should be written");

        let error = coordinator
            .room_activated(&doc_id)
            .expect_err("active lease from another node should reject acquisition");
        assert_eq!(
            error.to_string(),
            format!(
                "room coordination failed: document `{doc_id}` is already leased by node `node-b`"
            )
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn room_coordinator_from_config_rejects_invalid_file_timing() {
        let mut config = test_config("file");
        config.room_coordinator_heartbeat_interval_secs = 30;
        config.room_coordinator_lease_ttl_secs = 30;

        let error = match room_coordinator_from_config(&config) {
            Ok(_) => panic!("invalid file lease timing should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "failed to initialize file room coordinator: room coordination failed: ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS must be smaller than ROOM_COORDINATOR_LEASE_TTL_SECS when ROOM_COORDINATOR=file"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }

    #[test]
    fn room_coordinator_from_config_rejects_invalid_sqlite_timing() {
        let mut config = test_config("sqlite");
        config.room_coordinator_heartbeat_interval_secs = 30;
        config.room_coordinator_lease_ttl_secs = 30;

        let error = match room_coordinator_from_config(&config) {
            Ok(_) => panic!("invalid sqlite lease timing should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "failed to initialize sqlite room coordinator: room coordination failed: ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS must be smaller than ROOM_COORDINATOR_LEASE_TTL_SECS when ROOM_COORDINATOR=sqlite"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }

    #[test]
    fn room_coordinator_from_config_rejects_invalid_node_base_url() {
        let mut config = test_config("file");
        config.node_base_url = Some("http://node-a.internal/path".to_owned());

        let error = match room_coordinator_from_config(&config) {
            Ok(_) => panic!("invalid node base url should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "failed to initialize file room coordinator: room coordination failed: NODE_BASE_URL must be an origin-only absolute http/https URL without path/query, received `http://node-a.internal/path`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }

    #[test]
    fn room_coordinator_from_config_rejects_managed_mode_without_base_url() {
        let error = match room_coordinator_from_config(&test_config("managed")) {
            Ok(_) => panic!("managed mode without base url should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_COORDINATION_MANAGED_BASE_URL is required when ROOM_COORDINATOR=managed"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }

    #[test]
    fn room_coordinator_from_config_rejects_unknown_mode() {
        let error = match room_coordinator_from_config(&test_config("unsupported")) {
            Ok(_) => panic!("unknown room coordinator mode should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_COORDINATOR must be `noop`, `logging`, `file`, `sqlite`, or `managed`, received `unsupported`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }
}
