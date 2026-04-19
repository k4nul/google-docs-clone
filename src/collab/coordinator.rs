use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RoomCoordinatorError {
    #[error("room coordination failed: {0}")]
    Operation(String),
}

pub trait RoomCoordinator: Send + Sync {
    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError>;

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError>;
}

#[derive(Debug, Default)]
pub struct NoopRoomCoordinator;

impl RoomCoordinator for NoopRoomCoordinator {
    fn room_activated(&self, _doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        Ok(())
    }

    fn room_deactivated(&self, _doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        Ok(())
    }
}

pub fn noop_room_coordinator() -> Arc<dyn RoomCoordinator> {
    Arc::new(NoopRoomCoordinator)
}
