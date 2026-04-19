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
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    config::{Config, normalize_origin_url},
    errors::{AppError, AppResult},
};

#[derive(Debug, Error)]
pub enum RoomCoordinatorError {
    #[error("room coordination failed: {0}")]
    Operation(String),
}

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
    heartbeats: Mutex<HashMap<Uuid, FileRoomLeaseHeartbeat>>,
}

struct FileRoomLeaseHeartbeat {
    lease_id: Uuid,
    epoch: u64,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
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
    ) -> Result<FileRoomLeaseHeartbeat, RoomCoordinatorError> {
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

        Ok(FileRoomLeaseHeartbeat {
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
        other => Err(AppError::Config(format!(
            "ROOM_COORDINATOR must be `noop`, `logging`, or `file`, received `{other}`"
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

    fn test_config(room_coordinator: &str) -> Config {
        Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
            room_locator: "local".to_owned(),
            room_coordinator: room_coordinator.to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            room_coordinator_heartbeat_interval_secs: 1,
            room_coordinator_lease_ttl_secs: 5,
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
    fn room_coordinator_from_config_rejects_unknown_mode() {
        let error = match room_coordinator_from_config(&test_config("unsupported")) {
            Ok(_) => panic!("unknown room coordinator mode should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_COORDINATOR must be `noop`, `logging`, or `file`, received `unsupported`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }
}
