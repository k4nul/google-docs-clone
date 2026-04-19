use std::sync::Arc;

use axum::http::Uri;

use crate::{
    collab::coordinator::{RoomCoordinator, noop_room_coordinator, room_coordinator_from_config},
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
    room_coordinator: Arc<dyn RoomCoordinator>,
}

impl AppState {
    pub fn new(frontend_origin: impl Into<String>, api_token: impl Into<String>) -> Self {
        Self::with_snapshot_store_locator_and_coordinator(
            frontend_origin,
            api_token,
            in_memory_snapshot_store(),
            local_room_locator(),
            noop_room_coordinator(),
        )
        .expect("default in-memory state should initialize")
    }

    pub fn with_snapshot_store(
        frontend_origin: impl Into<String>,
        api_token: impl Into<String>,
        snapshot_store: Arc<dyn SnapshotStore>,
    ) -> AppResult<Self> {
        Self::with_snapshot_store_locator_and_coordinator(
            frontend_origin,
            api_token,
            snapshot_store,
            local_room_locator(),
            noop_room_coordinator(),
        )
    }

    pub fn with_snapshot_store_and_locator(
        frontend_origin: impl Into<String>,
        api_token: impl Into<String>,
        snapshot_store: Arc<dyn SnapshotStore>,
        room_locator: Arc<dyn RoomLocator>,
    ) -> AppResult<Self> {
        Self::with_snapshot_store_locator_and_coordinator(
            frontend_origin,
            api_token,
            snapshot_store,
            room_locator,
            noop_room_coordinator(),
        )
    }

    pub fn with_snapshot_store_locator_and_coordinator(
        frontend_origin: impl Into<String>,
        api_token: impl Into<String>,
        snapshot_store: Arc<dyn SnapshotStore>,
        room_locator: Arc<dyn RoomLocator>,
        room_coordinator: Arc<dyn RoomCoordinator>,
    ) -> AppResult<Self> {
        Self::with_snapshot_store_locator_and_coordinator_and_hydration(
            frontend_origin,
            api_token,
            snapshot_store,
            room_locator,
            room_coordinator,
            true,
        )
    }

    fn with_snapshot_store_locator_and_coordinator_and_hydration(
        frontend_origin: impl Into<String>,
        api_token: impl Into<String>,
        snapshot_store: Arc<dyn SnapshotStore>,
        room_locator: Arc<dyn RoomLocator>,
        room_coordinator: Arc<dyn RoomCoordinator>,
        hydrate_rooms_on_startup: bool,
    ) -> AppResult<Self> {
        let rooms = Arc::new(RoomRegistry::new(snapshot_store));
        if hydrate_rooms_on_startup {
            let hydrated_rooms = rooms
                .hydrate_from_store()
                .map_err(anyhow::Error::from)
                .map_err(AppError::from)?;

            tracing::info!(
                hydrated_rooms,
                "initialized room registry from snapshot store"
            );
        } else {
            tracing::info!(
                "skipped eager room hydration because distributed room ownership mode is enabled"
            );
        }

        Ok(Self {
            rooms,
            frontend_origin: Arc::<str>::from(frontend_origin.into()),
            api_token: Arc::<str>::from(api_token.into()),
            room_locator,
            room_coordinator,
        })
    }

    pub fn from_config(config: &Config) -> AppResult<Self> {
        let hydrate_rooms_on_startup = startup_room_hydration_enabled(config);
        Self::with_snapshot_store_locator_and_coordinator_and_hydration(
            config.frontend_origin.clone(),
            config.api_token.clone(),
            snapshot_store_from_config(config)
                .map_err(anyhow::Error::from)
                .map_err(AppError::from)?,
            room_locator_from_config(config)?,
            room_coordinator_from_config(config)?,
            hydrate_rooms_on_startup,
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

    pub fn room_coordinator(&self) -> Arc<dyn RoomCoordinator> {
        Arc::clone(&self.room_coordinator)
    }

    pub fn ensure_local_room_owner_for_request(
        &self,
        doc_id: &uuid::Uuid,
        request_uri: &Uri,
    ) -> AppResult<()> {
        self.ensure_local_room_owner(doc_id)
            .map_err(|error| error.with_redirect_from_request(request_uri))
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
                    redirect_url: None,
                })
            }
            Err(error) => {
                tracing::error!(doc_id = %doc_id, %error, "room ownership lookup failed");
                Err(AppError::from(anyhow::Error::from(error)))
            }
        }
    }
}

fn startup_room_hydration_enabled(config: &Config) -> bool {
    matches!(config.room_locator.trim(), "local")
        && matches!(config.room_coordinator.trim(), "noop" | "logging")
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
