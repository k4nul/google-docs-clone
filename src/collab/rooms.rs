use std::sync::Arc;

use dashmap::{DashMap, mapref::entry::Entry};
use tokio::sync::{OnceCell, RwLock};
use uuid::Uuid;
use yrs::{Doc, sync::Awareness};
use yrs_axum::{AwarenessRef, broadcast::BroadcastGroup};

use crate::models::document::Document;

pub struct Room {
    document: Document,
    awareness: AwarenessRef,
    broadcast_group: OnceCell<Arc<BroadcastGroup>>,
}

impl Room {
    pub fn new(doc_id: Uuid) -> Self {
        let awareness = Arc::new(RwLock::new(Awareness::new(Doc::new())));

        Self {
            document: Document::placeholder(doc_id),
            awareness,
            broadcast_group: OnceCell::new(),
        }
    }

    pub fn document(&self) -> Document {
        self.document.clone()
    }

    pub fn awareness(&self) -> AwarenessRef {
        self.awareness.clone()
    }

    pub async fn broadcast_group(&self) -> Arc<BroadcastGroup> {
        self.broadcast_group
            .get_or_init(|| async { Arc::new(BroadcastGroup::new(self.awareness(), 32).await) })
            .await
            .clone()
    }
}

#[derive(Default)]
pub struct RoomRegistry {
    rooms: DashMap<Uuid, Arc<Room>>,
}

impl RoomRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, doc_id: &Uuid) -> Option<Arc<Room>> {
        self.rooms.get(doc_id).map(|room| room.clone())
    }

    pub fn get_or_create(&self, doc_id: Uuid) -> Arc<Room> {
        match self.rooms.entry(doc_id) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let room = Arc::new(Room::new(doc_id));
                entry.insert(room.clone());
                room
            }
        }
    }

    pub fn list_documents(&self) -> Vec<Document> {
        let mut documents = self
            .rooms
            .iter()
            .map(|entry| entry.value().document())
            .collect::<Vec<_>>();

        documents.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.id.cmp(&right.id))
        });

        documents
    }
}
