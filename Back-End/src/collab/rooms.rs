use std::sync::{
    Arc, RwLock as StdRwLock,
    atomic::{AtomicUsize, Ordering},
};

use dashmap::{DashMap, mapref::entry::Entry};
use std::collections::BTreeMap;
use tokio::sync::{OnceCell, RwLock};
use uuid::Uuid;
use yrs::{
    Doc, ReadTxn, StateVector, Subscription, Transact, Update, sync::Awareness,
    updates::decoder::Decode,
};
use yrs_axum::{AwarenessRef, broadcast::BroadcastGroup};

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, SnapshotStore, StorageError, in_memory_snapshot_store},
};

pub struct Room {
    document: Arc<StdRwLock<Document>>,
    awareness: AwarenessRef,
    broadcast_group: OnceCell<Arc<BroadcastGroup>>,
    active_sessions: AtomicUsize,
    _update_persistence: Subscription,
}

impl Room {
    pub fn new(document: Document, snapshot_store: Arc<dyn SnapshotStore>) -> Self {
        let document = Arc::new(StdRwLock::new(document));
        let awareness = Awareness::new(Doc::new());
        let update_persistence =
            Self::observe_updates(document.clone(), snapshot_store, awareness.doc());
        let awareness = Arc::new(RwLock::new(awareness));

        Self {
            document,
            awareness,
            broadcast_group: OnceCell::new(),
            active_sessions: AtomicUsize::new(0),
            _update_persistence: update_persistence,
        }
    }

    pub fn from_snapshot(
        snapshot: DocumentSnapshot,
        snapshot_store: Arc<dyn SnapshotStore>,
    ) -> Result<Self, StorageError> {
        let document_id = snapshot.document.id;
        let document = Arc::new(StdRwLock::new(snapshot.document));
        let awareness = Awareness::new(Doc::new());
        {
            let mut txn = awareness.doc().transact_mut();
            let update = Update::decode_v1(snapshot.update.as_slice())
                .map_err(|_| StorageError::CorruptSnapshot(document_id))?;
            txn.apply_update(update);
        }
        let update_persistence =
            Self::observe_updates(document.clone(), snapshot_store, awareness.doc());
        let awareness = Arc::new(RwLock::new(awareness));

        Ok(Self {
            document,
            awareness,
            broadcast_group: OnceCell::new(),
            active_sessions: AtomicUsize::new(0),
            _update_persistence: update_persistence,
        })
    }

    fn observe_updates(
        document: Arc<StdRwLock<Document>>,
        snapshot_store: Arc<dyn SnapshotStore>,
        doc: &Doc,
    ) -> Subscription {
        doc.observe_update_v1(move |txn, _event| {
            let mut document = document
                .write()
                .expect("room document lock should not be poisoned");
            document.touch();
            let update = txn.encode_state_as_update_v1(&StateVector::default());
            let snapshot = DocumentSnapshot::new(document.clone(), update);
            drop(document);

            let doc_id = snapshot.document.id;
            if let Err(error) = snapshot_store.save_snapshot(snapshot) {
                tracing::warn!(
                    %doc_id,
                    %error,
                    "failed to persist snapshot after Yrs document update"
                );
            }
        })
        .expect("room Yrs document should allow update observer registration")
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

    pub fn rename_document(&self, title: String) -> Document {
        self.update_document(Some(title), None)
    }

    pub fn update_document(&self, title: Option<String>, hide_preview: Option<bool>) -> Document {
        let mut document = self
            .document
            .write()
            .expect("room document lock should not be poisoned");
        if let Some(title) = title {
            document.rename(title);
        }
        if let Some(hide_preview) = hide_preview {
            document.set_hide_preview(hide_preview);
        }
        document.clone()
    }

    pub fn snapshot(&self) -> Result<DocumentSnapshot, StorageError> {
        self.snapshot_with_touch(true)
    }

    pub fn catalog_snapshot(&self) -> Result<DocumentSnapshot, StorageError> {
        self.snapshot_with_touch(false)
    }

    fn snapshot_with_touch(&self, touch_document: bool) -> Result<DocumentSnapshot, StorageError> {
        let mut document = self
            .document
            .read()
            .expect("room document lock should not be poisoned")
            .clone();
        if touch_document {
            document.touch();
        }

        let awareness = self
            .awareness
            .try_read()
            .map_err(|_| StorageError::Busy(document.id))?;
        let update = awareness
            .doc()
            .transact()
            .encode_state_as_update_v1(&StateVector::default());

        if touch_document {
            *self
                .document
                .write()
                .expect("room document lock should not be poisoned") = document.clone();
        }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionTeardown {
    pub remaining_sessions: usize,
    pub evicted: bool,
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
        let room = Arc::new(Room::new(
            document.clone(),
            Arc::clone(&self.snapshot_store),
        ));
        self.snapshot_store.save_snapshot(room.snapshot()?)?;

        self.rooms.insert(document.id, room);

        Ok(document)
    }

    pub fn delete_document(&self, doc_id: &Uuid) -> Result<Option<Document>, StorageError> {
        if let Some(room) = self.get(doc_id) {
            if room.active_sessions() > 0 {
                return Err(StorageError::DocumentBusy(*doc_id));
            }
        }

        let document = self.rooms.remove(doc_id).map(|(_, room)| room.document());
        self.snapshot_store.delete_snapshot(doc_id)?;
        Ok(document)
    }

    pub fn rename_document(
        &self,
        doc_id: &Uuid,
        title: String,
    ) -> Result<Option<Document>, StorageError> {
        self.update_document(doc_id, Some(title), None)
    }

    pub fn update_document(
        &self,
        doc_id: &Uuid,
        title: Option<String>,
        hide_preview: Option<bool>,
    ) -> Result<Option<Document>, StorageError> {
        let Some(room) = self.get_or_restore(doc_id)? else {
            return Ok(None);
        };

        room.update_document(title, hide_preview);
        let snapshot = room.snapshot()?;
        let document = snapshot.document.clone();
        self.snapshot_store.save_snapshot(snapshot)?;

        Ok(Some(document))
    }

    pub fn get_or_create(&self, document: Document) -> Arc<Room> {
        match self.rooms.entry(document.id) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let room = Arc::new(Room::new(document, Arc::clone(&self.snapshot_store)));
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

        let room = Arc::new(Room::from_snapshot(
            snapshot,
            Arc::clone(&self.snapshot_store),
        )?);
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
    ) -> Result<SessionTeardown, StorageError> {
        let remaining_sessions = room.end_session();
        if remaining_sessions > 0 {
            return Ok(SessionTeardown {
                remaining_sessions,
                evicted: false,
            });
        }

        match self.rooms.entry(*doc_id) {
            Entry::Occupied(entry) if Arc::ptr_eq(entry.get(), room) => {
                self.snapshot_store.save_snapshot(room.snapshot()?)?;

                if room.active_sessions() == 0 {
                    entry.remove();
                    Ok(SessionTeardown {
                        remaining_sessions: 0,
                        evicted: true,
                    })
                } else {
                    Ok(SessionTeardown {
                        remaining_sessions: room.active_sessions(),
                        evicted: false,
                    })
                }
            }
            _ => Ok(SessionTeardown {
                remaining_sessions: room.active_sessions(),
                evicted: false,
            }),
        }
    }

    pub fn list_document_snapshots(&self) -> Result<Vec<DocumentSnapshot>, StorageError> {
        let mut snapshots = self
            .snapshot_store
            .list_snapshots()?
            .into_iter()
            .map(|snapshot| (snapshot.document.id, snapshot))
            .collect::<BTreeMap<_, _>>();

        for entry in self.rooms.iter() {
            let snapshot = entry.value().catalog_snapshot()?;
            snapshots.insert(snapshot.document.id, snapshot);
        }

        let mut snapshots = snapshots.into_values().collect::<Vec<_>>();

        snapshots.sort_by(|left, right| {
            left.document
                .created_at
                .cmp(&right.document.created_at)
                .then_with(|| left.document.id.cmp(&right.document.id))
        });

        Ok(snapshots)
    }

    pub fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let documents = self
            .list_document_snapshots()?
            .into_iter()
            .map(|snapshot| snapshot.document)
            .collect();

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
    fn registry_persists_document_updates_before_room_teardown() {
        let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
        let registry = RoomRegistry::new(snapshot_store.clone());
        let document = registry
            .create_document(Some("Autosaved".to_owned()))
            .expect("document should be created");
        let room = registry
            .get(&document.id)
            .expect("created document should have an active room");

        let awareness = room.awareness();
        {
            let doc = awareness.blocking_write().doc().clone();
            let text = doc.get_or_insert_text("content");
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, "saved while active");
        }

        let restored_registry = RoomRegistry::new(snapshot_store);
        let restored_room = restored_registry
            .get_or_restore(&document.id)
            .expect("snapshot lookup should succeed")
            .expect("document should restore from autosaved snapshot");

        let restored_doc = restored_room.awareness().blocking_read().doc().clone();
        let restored_text = restored_doc.get_or_insert_text("content");
        let restored_value = restored_text.get_string(&restored_doc.transact());

        assert_eq!(restored_value, "saved while active");
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

        let teardown = registry
            .persist_and_evict_if_idle(&document.id, &room)
            .expect("idle room eviction should succeed");

        assert!(teardown.evicted);
        assert_eq!(teardown.remaining_sessions, 0);
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

        let teardown = registry
            .persist_and_evict_if_idle(&document.id, &room)
            .expect("room release should succeed");

        assert!(!teardown.evicted);
        assert_eq!(teardown.remaining_sessions, 1);
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
        let teardown = registry
            .persist_and_evict_if_idle(&document.id, &room)
            .expect("idle room eviction should succeed");

        assert!(teardown.evicted);
        assert_eq!(teardown.remaining_sessions, 0);
        assert!(registry.get(&document.id).is_none());

        let documents = registry
            .list_documents()
            .expect("document catalog should load from snapshot store");

        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id, document.id);
        assert_eq!(documents[0].title, "Persisted catalog");
    }

    #[test]
    fn registry_rejects_delete_while_room_has_active_sessions() {
        let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
        let registry = RoomRegistry::new(snapshot_store.clone());
        let document = registry
            .create_document(Some("Busy room".to_owned()))
            .expect("document should be created");
        let room = registry
            .get(&document.id)
            .expect("created document should have an active room");

        assert_eq!(room.start_session(), 1);

        let error = registry
            .delete_document(&document.id)
            .expect_err("delete should fail while sessions are active");
        assert!(matches!(error, StorageError::DocumentBusy(id) if id == document.id));
        assert!(registry.get(&document.id).is_some());
        assert!(
            snapshot_store
                .load_snapshot(&document.id)
                .expect("snapshot lookup should succeed")
                .is_some()
        );
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
        let teardown = registry
            .persist_and_evict_if_idle(&document.id, &room)
            .expect("idle room eviction should succeed");
        assert!(teardown.evicted);
        assert_eq!(teardown.remaining_sessions, 0);

        let restored_registry = RoomRegistry::new(snapshot_store);
        let hydrated = restored_registry
            .hydrate_from_store()
            .expect("startup hydration should succeed");

        assert_eq!(hydrated, 1);
        assert!(restored_registry.get(&document.id).is_some());
    }
}
