use std::sync::Arc;

use thiserror::Error;
use tracing::info;
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

pub fn noop_room_coordinator() -> Arc<dyn RoomCoordinator> {
    Arc::new(NoopRoomCoordinator)
}

pub fn logging_room_coordinator(node_id: impl Into<String>) -> Arc<dyn RoomCoordinator> {
    Arc::new(LoggingRoomCoordinator::new(node_id))
}

pub fn room_coordinator_from_config(config: &Config) -> AppResult<Arc<dyn RoomCoordinator>> {
    match config.room_coordinator.trim().to_ascii_lowercase().as_str() {
        "noop" => Ok(noop_room_coordinator()),
        "logging" => Ok(logging_room_coordinator(config.node_id.clone())),
        other => Err(AppError::Config(format!(
            "ROOM_COORDINATOR must be `noop` or `logging`, received `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn room_coordinator_from_config_rejects_unknown_mode() {
        let error = match room_coordinator_from_config(&test_config("unsupported")) {
            Ok(_) => panic!("unknown room coordinator mode should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_COORDINATOR must be `noop` or `logging`, received `unsupported`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }
}
