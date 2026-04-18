use std::sync::{
    Arc, RwLock as StdRwLock,
    atomic::{AtomicUsize, Ordering},
};

use dashmap::{DashMap, mapref::entry::Entry};
use std::collections::BTreeMap;
use tokio::sync::{OnceCell, RwLock};
use uuid::Uuid;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update, sync::Awareness, updates::decoder::Decode};
use yrs_axum::{AwarenessRef, broadcast::BroadcastGroup};

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, SnapshotStore, StorageError, in_memory_snapshot_store},
};

pub struct Room {
    document: StdRwLock<Document>,
    awareness: AwarenessRef,
    broadcast_group: OnceCell<Arc<BroadcastGroup>>,
    active_sessions: AtomicUsize,
}

impl Room {
    pub fn new(document: Document) -> Self {
        let awareness = Arc::new(RwLock::new(Awareness::new(Doc::new())));

        Self {
            document: StdRwLock::new(document),
            awareness,
            broadcast_group: OnceCell::new(),
            active_sessions: AtomicUsize::new(0),
        }
    }

    pub fn from_snapshot(snapshot: DocumentSnapshot) -> Result<Self, StorageError> {
        let mut awareness = Arc::new(RwLock::new(Awareness::new(Doc::new())));
        {
            let awareness_ref = Arc::get_mut(&mut awareness)
                .expect("newly created awareness should not have other references");
            let awareness_guard = awareness_ref.get_mut();
            let mut txn = awareness_guard.doc().transact_mut();
            let update = Update::decode_v1(snapshot.update.as_slice())
                .map_err(|_| StorageError::CorruptSnapshot(snapshot.document.id))?;
            txn.apply_update(update);
        }

        Ok(Self {
            document: StdRwLock::new(snapshot.document),
            awareness,
            broadcast_group: OnceCell::new(),
            active_sessions: AtomicUsize::new(0),
        })
    }

    pub fn document(&self) -> Document {
        self.document
            .read()
            .expect("room document lock should not be poisoned")
            .clone()
    }

    pub fn authorizes(&self, token: &str) -> bool {
        self.document
            .read()
            .expect("room document lock should not be poisoned")
            .authorize(token)
    }

    pub fn snapshot(&self) -> Result<DocumentSnapshot, StorageError> {
        let mut document = self
            .document
            .read()
            .expect("room document lock should not be poisoned")
            .clone();
        document.touch();

        let awareness = self
            .awareness
            .try_read()
            .map_err(|_| StorageError::Busy(document.id))?;
        let update = awareness
            .doc()
            .transact()
            .encode_state_as_update_v1(&StateVector::default());

        *self
            .document
            .write()
            .expect("room document lock should not be poisoned") = document.clone();

        Ok(DocumentSnapshot::new(document, update))
    }

    pub fn awareness(&self) -> AwarenessRef {
        self.awareness.clone()
    }

    pub fn active_sessions(&self) -> usize {
        self.active_sessions.load(Ordering::SeqCst)
    }

    pub fn start_session(&self) -> usize {
        self.active_sessions.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn end_session(&self) -> usize {
        self.active_sessions
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |active| {
                active.checked_sub(1)
            })
            .expect("room session count should not underflow")
            - 1
    }

    pub async fn broadcast_group(&self) -> Arc<BroadcastGroup> {
        self.broadcast_group
            .get_or_init(|| async { Arc::new(BroadcastGroup::new(self.awareness(), 32).await) })
            .await
            .clone()
    }
}

pub struct RoomRegistry {
    rooms: DashMap<Uuid, Arc<Room>>,
    snapshot_store: Arc<dyn SnapshotStore>,
}

impl RoomRegistry {
    pub fn new(snapshot_store: Arc<dyn SnapshotStore>) -> Self {
        Self {
            rooms: DashMap::new(),
            snapshot_store,
        }
    }

    pub fn get(&self, doc_id: &Uuid) -> Option<Arc<Room>> {
        self.rooms.get(doc_id).map(|room| room.clone())
    }

    pub fn create_document(&self, title: Option<String>) -> Result<Document, StorageError> {
        let document = Document::new(Uuid::new_v4(), title);
        let room = Arc::new(Room::new(document.clone()));
        self.snapshot_store.save_snapshot(room.snapshot()?)?;

        self.rooms.insert(document.id, room);

        Ok(document)
    }

    pub fn delete_document(&self, doc_id: &Uuid) -> Result<Option<Document>, StorageError> {
        let document = self.rooms.remove(doc_id).map(|(_, room)| room.document());
        self.snapshot_store.delete_snapshot(doc_id)?;
        Ok(document)
    }

    pub fn get_or_create(&self, document: Document) -> Arc<Room> {
        match self.rooms.entry(document.id) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let room = Arc::new(Room::new(document));
                entry.insert(room.clone());
                room
            }
        }
    }

    pub fn get_or_restore(&self, doc_id: &Uuid) -> Result<Option<Arc<Room>>, StorageError> {
        if let Some(room) = self.get(doc_id) {
            return Ok(Some(room));
        }

        let Some(snapshot) = self.snapshot_store.load_snapshot(doc_id)? else {
            return Ok(None);
        };

        let room = Arc::new(Room::from_snapshot(snapshot)?);
        let room_id = room.document().id;

        match self.rooms.entry(room_id) {
            Entry::Occupied(entry) => Ok(Some(entry.get().clone())),
            Entry::Vacant(entry) => {
                entry.insert(room.clone());
                Ok(Some(room))
            }
        }
    }

    pub fn persist_and_evict_if_idle(
        &self,
        doc_id: &Uuid,
        room: &Arc<Room>,
    ) -> Result<bool, StorageError> {
        let remaining_sessions = room.end_session();
        if remaining_sessions > 0 {
            return Ok(false);
        }

        match self.rooms.entry(*doc_id) {
            Entry::Occupied(entry) if Arc::ptr_eq(entry.get(), room) => {
                self.snapshot_store.save_snapshot(room.snapshot()?)?;

                if room.active_sessions() == 0 {
                    entry.remove();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    pub fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = self
            .snapshot_store
            .list_documents()?
            .into_iter()
            .map(|document| (document.id, document))
            .collect::<BTreeMap<_, _>>();

        for entry in self.rooms.iter() {
            let document = entry.value().document();
            documents.insert(document.id, document);
        }

        let mut documents = documents.into_values().collect::<Vec<_>>();

        documents.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(documents)
    }

    pub fn hydrate_from_store(&self) -> Result<usize, StorageError> {
        let mut hydrated = 0;

        for document in self.snapshot_store.list_documents()? {
            if self.get_or_restore(&document.id)?.is_some() {
                hydrated += 1;
            }
        }

        Ok(hydrated)
    }
}

impl Default for RoomRegistry {
    fn default() -> Self {
        Self::new(in_memory_snapshot_store())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{InMemorySnapshotStore, SnapshotStore};
    use yrs::{GetString, Text};

    #[test]
    fn registry_restores_document_from_snapshot_store() {
        let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
        let registry = RoomRegistry::new(snapshot_store.clone());
        let document = registry
            .create_document(Some("Recovered".to_owned()))
            .expect("document should be created");
        let room = registry
            .get(&document.id)
            .expect("created document should have an active room");

        let awareness = room.awareness();
        {
            let doc = awareness.blocking_write().doc().clone();
            let text = doc.get_or_insert_text("content");
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, "hello world");
        }

        snapshot_store
            .save_snapshot(room.snapshot().expect("snapshot should be captured"))
            .expect("snapshot should be persisted");

        let restored_registry = RoomRegistry::new(snapshot_store);
        let restored_room = restored_registry
            .get_or_restore(&document.id)
            .expect("snapshot lookup should succeed")
            .expect("document should restore from snapshot");

        let restored_doc = restored_room.awareness().blocking_read().doc().clone();
        let restored_text = restored_doc.get_or_insert_text("content");
        let restored_value = restored_text.get_string(&restored_doc.transact());

        assert_eq!(restored_room.document().id, document.id);
        assert_eq!(restored_value, "hello world");
    }

    #[test]
    fn registry_evicts_idle_room_after_snapshot_persist() {
        let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
        let registry = RoomRegistry::new(snapshot_store.clone());
        let document = registry
            .create_document(Some("Evicted".to_owned()))
            .expect("document should be created");
        let room = registry
            .get(&document.id)
            .expect("created document should have an active room");

        let awareness = room.awareness();
        {
            let doc = awareness.blocking_write().doc().clone();
            let text = doc.get_or_insert_text("content");
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, "persist me");
        }

        assert_eq!(room.start_session(), 1);

        let evicted = registry
            .persist_and_evict_if_idle(&document.id, &room)
            .expect("idle room eviction should succeed");

        assert!(evicted);
        assert!(registry.get(&document.id).is_none());

        let restored_room = registry
            .get_or_restore(&document.id)
            .expect("snapshot lookup should succeed")
            .expect("document should restore from snapshot");
        let restored_doc = restored_room.awareness().blocking_read().doc().clone();
        let restored_text = restored_doc.get_or_insert_text("content");
        let restored_value = restored_text.get_string(&restored_doc.transact());

        assert_eq!(restored_value, "persist me");
    }

    #[test]
    fn registry_keeps_room_while_other_sessions_are_active() {
        let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
        let registry = RoomRegistry::new(snapshot_store);
        let document = registry
            .create_document(Some("Shared".to_owned()))
            .expect("document should be created");
        let room = registry
            .get(&document.id)
            .expect("created document should have an active room");

        assert_eq!(room.start_session(), 1);
        assert_eq!(room.start_session(), 2);

        let evicted = registry
            .persist_and_evict_if_idle(&document.id, &room)
            .expect("room release should succeed");

        assert!(!evicted);
        assert!(registry.get(&document.id).is_some());
        assert_eq!(room.active_sessions(), 1);
    }

    #[test]
    fn registry_lists_documents_from_snapshot_store_after_eviction() {
        let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
        let registry = RoomRegistry::new(snapshot_store);
        let document = registry
            .create_document(Some("Persisted catalog".to_owned()))
            .expect("document should be created");
        let room = registry
            .get(&document.id)
            .expect("created document should have an active room");

        assert_eq!(room.start_session(), 1);
        let evicted = registry
            .persist_and_evict_if_idle(&document.id, &room)
            .expect("idle room eviction should succeed");

        assert!(evicted);
        assert!(registry.get(&document.id).is_none());

        let documents = registry
            .list_documents()
            .expect("document catalog should load from snapshot store");

        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id, document.id);
        assert_eq!(documents[0].title, "Persisted catalog");
    }

    #[test]
    fn registry_hydrates_rooms_from_snapshot_store_catalog() {
        let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
        let registry = RoomRegistry::new(snapshot_store.clone());
        let document = registry
            .create_document(Some("Hydrated room".to_owned()))
            .expect("document should be created");
        let room = registry
            .get(&document.id)
            .expect("created document should have an active room");

        assert_eq!(room.start_session(), 1);
        let evicted = registry
            .persist_and_evict_if_idle(&document.id, &room)
            .expect("idle room eviction should succeed");
        assert!(evicted);

        let restored_registry = RoomRegistry::new(snapshot_store);
        let hydrated = restored_registry
            .hydrate_from_store()
            .expect("startup hydration should succeed");

        assert_eq!(hydrated, 1);
        assert!(restored_registry.get(&document.id).is_some());
    }
}
