use std::sync::Arc;

use crate::{
    collab::locator::{ResolvedRoom, RoomLocator, local_room_locator, room_locator_from_config},
    collab::rooms::RoomRegistry,
    config::{Config, DEFAULT_FRONTEND_ORIGIN},
    errors::{AppError, AppResult},
    storage::{SnapshotStore, in_memory_snapshot_store, snapshot_store_from_config},
};

#[derive(Clone)]
pub struct AppState {
    rooms: Arc<RoomRegistry>,
    frontend_origin: Arc<str>,
    api_token: Arc<str>,
    room_locator: Arc<dyn RoomLocator>,
}

impl AppState {
    pub fn new(frontend_origin: impl Into<String>, api_token: impl Into<String>) -> Self {
        Self::with_snapshot_store_and_locator(
            frontend_origin,
            api_token,
            in_memory_snapshot_store(),
            local_room_locator(),
        )
        .expect("default in-memory state should initialize")
    }

    pub fn with_snapshot_store(
        frontend_origin: impl Into<String>,
        api_token: impl Into<String>,
        snapshot_store: Arc<dyn SnapshotStore>,
    ) -> AppResult<Self> {
        Self::with_snapshot_store_and_locator(
            frontend_origin,
            api_token,
            snapshot_store,
            local_room_locator(),
        )
    }

    pub fn with_snapshot_store_and_locator(
        frontend_origin: impl Into<String>,
        api_token: impl Into<String>,
        snapshot_store: Arc<dyn SnapshotStore>,
        room_locator: Arc<dyn RoomLocator>,
    ) -> AppResult<Self> {
        let rooms = Arc::new(RoomRegistry::new(snapshot_store));
        let hydrated_rooms = rooms
            .hydrate_from_store()
            .map_err(anyhow::Error::from)
            .map_err(AppError::from)?;

        tracing::info!(
            hydrated_rooms,
            "initialized room registry from snapshot store"
        );

        Ok(Self {
            rooms,
            frontend_origin: Arc::<str>::from(frontend_origin.into()),
            api_token: Arc::<str>::from(api_token.into()),
            room_locator,
        })
    }

    pub fn from_config(config: &Config) -> AppResult<Self> {
        Self::with_snapshot_store_and_locator(
            config.frontend_origin.clone(),
            config.api_token.clone(),
            snapshot_store_from_config(config)
                .map_err(anyhow::Error::from)
                .map_err(AppError::from)?,
            room_locator_from_config(config)?,
        )
    }

    pub fn rooms(&self) -> &RoomRegistry {
        self.rooms.as_ref()
    }

    pub fn rooms_registry(&self) -> Arc<RoomRegistry> {
        Arc::clone(&self.rooms)
    }

    pub fn frontend_origin(&self) -> &str {
        &self.frontend_origin
    }

    pub fn api_token(&self) -> &str {
        &self.api_token
    }

    pub fn ensure_local_room_owner(&self, doc_id: &uuid::Uuid) -> AppResult<()> {
        match self.room_locator.resolve(doc_id) {
            Ok(ResolvedRoom::Local) => Ok(()),
            Ok(ResolvedRoom::Remote(owner)) => {
                tracing::warn!(
                    doc_id = %doc_id,
                    owner_node_id = owner.node_id.as_str(),
                    owner_base_url = owner.base_url.as_deref().unwrap_or("<unknown>"),
                    "rejected request for room owned by another collaboration node"
                );
                Err(AppError::RemoteOwner {
                    message: format!("document `{doc_id}` is owned by another collaboration node"),
                    owner_node_id: owner.node_id,
                    owner_base_url: owner.base_url,
                })
            }
            Err(error) => {
                tracing::error!(doc_id = %doc_id, %error, "room ownership lookup failed");
                Err(AppError::from(anyhow::Error::from(error)))
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::with_snapshot_store(
            DEFAULT_FRONTEND_ORIGIN,
            crate::config::DEFAULT_API_TOKEN,
            in_memory_snapshot_store(),
        )
        .expect("default app state should initialize room registry")
    }
}
