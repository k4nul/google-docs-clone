use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    config::Config,
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

#[derive(Debug, Clone)]
pub struct FileRoomCoordinator {
    node_id: Arc<str>,
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedRoomCoordinatorState {
    doc_id: Uuid,
    node_id: String,
    activated_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl FileRoomCoordinator {
    pub fn new(
        node_id: impl Into<String>,
        root: impl Into<PathBuf>,
    ) -> Result<Self, RoomCoordinatorError> {
        let node_id = node_id.into();
        let node_id = node_id.trim();
        if node_id.is_empty() {
            return Err(RoomCoordinatorError::Operation(
                "NODE_ID cannot be empty when ROOM_COORDINATOR=file".to_owned(),
            ));
        }

        let root = root.into();
        fs::create_dir_all(&root).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to create room coordinator state dir `{}`: {error}",
                root.display()
            ))
        })?;

        let coordinator = Self {
            node_id: Arc::<str>::from(node_id.to_owned()),
            root,
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
}

impl RoomCoordinator for FileRoomCoordinator {
    fn mode(&self) -> &'static str {
        "file"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        let timestamp = Utc::now();
        let state = PersistedRoomCoordinatorState {
            doc_id: *doc_id,
            node_id: self.node_id().to_owned(),
            activated_at: timestamp,
            updated_at: timestamp,
        };
        let path = self.state_path(doc_id);
        let bytes = serde_json::to_vec(&state).map_err(|error| {
            RoomCoordinatorError::Operation(format!(
                "failed to serialize room coordinator state `{}`: {error}",
                path.display()
            ))
        })?;

        self.write_state_atomically(doc_id, &path, &bytes)?;
        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            path = %path.display(),
            "persisted file-backed room coordinator state"
        );
        Ok(())
    }

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        let path = self.state_path(doc_id);
        self.remove_file_if_exists(&path)?;

        for temp_path in self.matching_temp_paths(doc_id)? {
            self.remove_file_if_exists(&temp_path)?;
        }

        info!(
            coordinator_mode = self.mode(),
            node_id = self.node_id(),
            %doc_id,
            path = %path.display(),
            "removed file-backed room coordinator state"
        );
        Ok(())
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
    root: impl Into<PathBuf>,
) -> Result<Arc<dyn RoomCoordinator>, RoomCoordinatorError> {
    Ok(Arc::new(FileRoomCoordinator::new(node_id, root)?))
}

pub fn room_coordinator_from_config(config: &Config) -> AppResult<Arc<dyn RoomCoordinator>> {
    match config.room_coordinator.trim().to_ascii_lowercase().as_str() {
        "noop" => Ok(noop_room_coordinator()),
        "logging" => Ok(logging_room_coordinator(config.node_id.clone())),
        "file" => file_room_coordinator(
            config.node_id.clone(),
            config.room_coordinator_state_dir.clone(),
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
    use std::{fs, path::PathBuf};

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
            room_locator: "local".to_owned(),
            room_coordinator: room_coordinator.to_owned(),
            room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
            node_id: "node-a".to_owned(),
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
        let coordinator = FileRoomCoordinator::new("node-a", &root)
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
        assert!(state.updated_at >= state.activated_at);

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
