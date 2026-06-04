use std::{fs, path::PathBuf, time::Duration};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use tracing::warn;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, SnapshotStore, StorageError},
};

pub struct SqliteSnapshotStore {
    path: PathBuf,
}

struct SnapshotRow {
    doc_id: String,
    title: String,
    created_at: String,
    updated_at: String,
    access_token: String,
    hide_preview: bool,
    update: Vec<u8>,
}

impl SqliteSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SQLITE_PATH cannot be empty when SNAPSHOT_STORE=sqlite".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = Self { path };
        let connection = store.open_connection()?;
        store.initialize_schema(&connection)?;
        Ok(store)
    }

    fn open_connection(&self) -> Result<Connection, StorageError> {
        let connection = Connection::open(&self.path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        Ok(connection)
    }

    fn initialize_schema(&self, connection: &Connection) -> Result<(), StorageError> {
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS snapshots (
                    doc_id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    access_token TEXT NOT NULL,
                    hide_preview INTEGER NOT NULL DEFAULT 0,
                    update_bytes BLOB NOT NULL
                );",
            )
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        if !self.table_has_column(connection, "snapshots", "hide_preview")? {
            connection
                .execute(
                    "ALTER TABLE snapshots
                     ADD COLUMN hide_preview INTEGER NOT NULL DEFAULT 0",
                    params![],
                )
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        }
        Ok(())
    }

    fn table_has_column(
        &self,
        connection: &Connection,
        table_name: &str,
        column_name: &str,
    ) -> Result<bool, StorageError> {
        let mut statement = connection
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let columns = statement
            .query_map(params![], |row| row.get::<_, String>(1))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        for column in columns {
            let column = column
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            if column == column_name {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn snapshot_from_row(&self, row: SnapshotRow) -> Result<DocumentSnapshot, StorageError> {
        let doc_id =
            Uuid::parse_str(&row.doc_id).map_err(|_| StorageError::CorruptSnapshot(Uuid::nil()))?;
        self.snapshot_from_row_for_doc_id(doc_id, row)
    }

    fn snapshot_from_row_for_doc_id(
        &self,
        expected_doc_id: Uuid,
        row: SnapshotRow,
    ) -> Result<DocumentSnapshot, StorageError> {
        let stored_doc_id = Uuid::parse_str(&row.doc_id)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        if stored_doc_id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        let created_at = parse_timestamp(&row.created_at, expected_doc_id)?;
        let updated_at = parse_timestamp(&row.updated_at, expected_doc_id)?;

        Ok(DocumentSnapshot::new(
            Document::from_parts(
                stored_doc_id,
                row.title,
                created_at,
                updated_at,
                row.access_token,
                row.hide_preview,
            ),
            row.update,
        ))
    }

    fn load_snapshot_row(
        &self,
        connection: &Connection,
        doc_id: &Uuid,
    ) -> Result<Option<SnapshotRow>, StorageError> {
        connection
            .query_row(
                "SELECT doc_id, title, created_at, updated_at, access_token, hide_preview, update_bytes
                 FROM snapshots
                 WHERE doc_id = ?1",
                [doc_id.to_string()],
                |row| {
                    Ok(SnapshotRow {
                        doc_id: row.get(0)?,
                        title: row.get(1)?,
                        created_at: row.get(2)?,
                        updated_at: row.get(3)?,
                        access_token: row.get(4)?,
                        hide_preview: row.get::<_, i64>(5)? != 0,
                        update: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for SqliteSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let connection = self.open_connection()?;
        let Some(row) = self.load_snapshot_row(&connection, doc_id)? else {
            return Ok(None);
        };

        self.snapshot_from_row_for_doc_id(*doc_id, row).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let connection = self.open_connection()?;
        self.initialize_schema(&connection)?;
        let document = snapshot.document;
        let access_token = document.access_token().to_owned();

        connection
            .execute(
                "INSERT INTO snapshots (
                    doc_id,
                    title,
                    created_at,
                    updated_at,
                    access_token,
                    hide_preview,
                    update_bytes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(doc_id) DO UPDATE SET
                    title = excluded.title,
                    created_at = excluded.created_at,
                    updated_at = excluded.updated_at,
                    access_token = excluded.access_token,
                    hide_preview = excluded.hide_preview,
                    update_bytes = excluded.update_bytes",
                params![
                    document.id.to_string(),
                    document.title,
                    document.created_at.to_rfc3339(),
                    document.updated_at.to_rfc3339(),
                    access_token,
                    if document.hide_preview { 1_i64 } else { 0_i64 },
                    snapshot.update,
                ],
            )
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let connection = self.open_connection()?;
        self.initialize_schema(&connection)?;
        connection
            .execute(
                "DELETE FROM snapshots WHERE doc_id = ?1",
                [doc_id.to_string()],
            )
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let connection = self.open_connection()?;
        self.initialize_schema(&connection)?;
        let mut statement = connection
            .prepare(
                "SELECT doc_id, title, created_at, updated_at, access_token, hide_preview, update_bytes
                 FROM snapshots
                 ORDER BY created_at ASC, doc_id ASC",
            )
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let rows = statement
            .query_map(params![], |row| {
                Ok(SnapshotRow {
                    doc_id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    access_token: row.get(4)?,
                    hide_preview: row.get::<_, i64>(5)? != 0,
                    update: row.get(6)?,
                })
            })
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        let mut documents = Vec::new();
        for row in rows {
            let row =
                row.map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            match self.snapshot_from_row(row) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt sqlite snapshot row while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}

fn parse_timestamp(value: &str, doc_id: Uuid) -> Result<DateTime<Utc>, StorageError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| StorageError::CorruptSnapshot(doc_id))
}
