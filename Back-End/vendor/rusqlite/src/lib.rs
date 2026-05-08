use std::{
    cell::Cell,
    collections::BTreeMap,
    error::Error as StdError,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

#[cfg(not(windows))]
use std::fs::File;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

const DEFAULT_BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serde(serde_json::Error),
    InvalidQuery(String),
    InvalidParameter(String),
    InvalidColumnIndex(usize),
    InvalidColumnType(&'static str),
    QueryReturnedNoRows,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::InvalidQuery(query) => write!(f, "unsupported sqlite shim query: {query}"),
            Self::InvalidParameter(message) => write!(f, "{message}"),
            Self::InvalidColumnIndex(index) => write!(f, "invalid column index `{index}`"),
            Self::InvalidColumnType(kind) => write!(f, "invalid column type for `{kind}`"),
            Self::QueryReturnedNoRows => f.write_str("query returned no rows"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::InvalidQuery(_)
            | Self::InvalidParameter(_)
            | Self::InvalidColumnIndex(_)
            | Self::InvalidColumnType(_)
            | Self::QueryReturnedNoRows => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionBehavior {
    Immediate,
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Integer(i64),
    Text(String),
    Blob(Vec<u8>),
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        Self::Text(value.clone())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Integer(i64::from(value))
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self::Blob(value)
    }
}

impl From<&Vec<u8>> for Value {
    fn from(value: &Vec<u8>) -> Self {
        Self::Blob(value.clone())
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Self::Null,
        }
    }
}

pub trait Params {
    fn into_values(self) -> Vec<Value>;
}

impl Params for Vec<Value> {
    fn into_values(self) -> Vec<Value> {
        self
    }
}

impl<T, const N: usize> Params for [T; N]
where
    T: Into<Value>,
{
    fn into_values(self) -> Vec<Value> {
        self.into_iter().map(Into::into).collect()
    }
}

#[macro_export]
macro_rules! params {
    () => {
        Vec::<$crate::Value>::new()
    };
    ($($value:expr),+ $(,)?) => {
        vec![$($crate::Value::from($value)),+]
    };
}

pub trait OptionalExtension<T> {
    fn optional(self) -> Result<Option<T>>;
}

impl<T> OptionalExtension<T> for Result<T> {
    fn optional(self) -> Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

pub struct Connection {
    path: PathBuf,
    busy_timeout: Cell<Duration>,
    last_changes: Cell<usize>,
}

impl Connection {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }

        Ok(Self {
            path,
            busy_timeout: Cell::new(DEFAULT_BUSY_TIMEOUT),
            last_changes: Cell::new(0),
        })
    }

    pub fn busy_timeout(&self, timeout: Duration) -> Result<()> {
        self.busy_timeout.set(timeout);
        Ok(())
    }

    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        let normalized = normalize_sql(sql);
        if normalized.starts_with("CREATE TABLE IF NOT EXISTS snapshots")
            || normalized.starts_with("CREATE TABLE IF NOT EXISTS room_leases")
        {
            self.last_changes.set(0);
            return Ok(());
        }

        Err(Error::InvalidQuery(normalized))
    }

    pub fn execute<P>(&self, sql: &str, params: P) -> Result<usize>
    where
        P: Params,
    {
        let _lock = acquire_file_lock(&self.path, self.busy_timeout.get())?;
        let mut db = load_db(&self.path)?;
        let changed = execute_mutation(&mut db, sql, params.into_values())?;
        if changed > 0 {
            persist_db(&self.path, &db)?;
        }
        self.last_changes.set(changed);
        Ok(changed)
    }

    pub fn query_row<P, F, T>(&self, sql: &str, params: P, f: F) -> Result<T>
    where
        P: Params,
        F: FnOnce(&Row<'_>) -> Result<T>,
    {
        let _lock = acquire_file_lock(&self.path, self.busy_timeout.get())?;
        let db = load_db(&self.path)?;
        let rows = query_rows(&db, sql, params.into_values())?;
        let values = rows.into_iter().next().ok_or(Error::QueryReturnedNoRows)?;
        let row = Row {
            values,
            _marker: PhantomData,
        };
        f(&row)
    }

    pub fn prepare<'conn>(&'conn self, sql: &str) -> Result<Statement<'conn>> {
        Ok(Statement {
            connection: self,
            sql: sql.to_owned(),
        })
    }

    pub fn transaction_with_behavior(
        &self,
        _behavior: TransactionBehavior,
    ) -> Result<Transaction<'_>> {
        let lock = acquire_file_lock(&self.path, self.busy_timeout.get())?;
        let state = load_db(&self.path)?;
        Ok(Transaction {
            connection: self,
            lock,
            state,
            last_changes: 0,
            committed: false,
        })
    }

    pub fn changes(&self) -> usize {
        self.last_changes.get()
    }
}

pub struct Transaction<'conn> {
    connection: &'conn Connection,
    lock: FileLock,
    state: PersistedDb,
    last_changes: usize,
    committed: bool,
}

impl<'conn> Transaction<'conn> {
    pub fn query_row<P, F, T>(&self, sql: &str, params: P, f: F) -> Result<T>
    where
        P: Params,
        F: FnOnce(&Row<'_>) -> Result<T>,
    {
        let rows = query_rows(&self.state, sql, params.into_values())?;
        let values = rows.into_iter().next().ok_or(Error::QueryReturnedNoRows)?;
        let row = Row {
            values,
            _marker: PhantomData,
        };
        f(&row)
    }

    pub fn execute<P>(&mut self, sql: &str, params: P) -> Result<usize>
    where
        P: Params,
    {
        let changed = execute_mutation(&mut self.state, sql, params.into_values())?;
        self.last_changes = changed;
        Ok(changed)
    }

    pub fn changes(&self) -> usize {
        self.last_changes
    }

    pub fn commit(mut self) -> Result<()> {
        let _ = &self.lock;
        if self.last_changes > 0 {
            persist_db(&self.connection.path, &self.state)?;
        }
        self.connection.last_changes.set(self.last_changes);
        self.committed = true;
        Ok(())
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.connection.last_changes.set(0);
        }
    }
}

pub struct Statement<'conn> {
    connection: &'conn Connection,
    sql: String,
}

impl<'conn> Statement<'conn> {
    pub fn query_map<P, F, T>(&mut self, params: P, mut f: F) -> Result<MappedRows<T>>
    where
        P: Params,
        F: FnMut(&Row<'_>) -> Result<T>,
    {
        let _lock = acquire_file_lock(&self.connection.path, self.connection.busy_timeout.get())?;
        let db = load_db(&self.connection.path)?;
        let rows = query_rows(&db, &self.sql, params.into_values())?;
        let mut mapped = Vec::with_capacity(rows.len());
        for values in rows {
            let row = Row {
                values,
                _marker: PhantomData,
            };
            mapped.push(f(&row));
        }

        Ok(MappedRows {
            inner: mapped.into_iter(),
        })
    }
}

pub struct MappedRows<T> {
    inner: std::vec::IntoIter<Result<T>>,
}

impl<T> Iterator for MappedRows<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct Row<'stmt> {
    values: Vec<Value>,
    _marker: PhantomData<&'stmt ()>,
}

pub trait RowIndex {
    fn as_usize(&self) -> usize;
}

impl RowIndex for usize {
    fn as_usize(&self) -> usize {
        *self
    }
}

pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self>;
}

impl FromValue for String {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Text(value) => Ok(value.clone()),
            _ => Err(Error::InvalidColumnType("String")),
        }
    }
}

impl FromValue for Option<String> {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Null => Ok(None),
            Value::Text(value) => Ok(Some(value.clone())),
            _ => Err(Error::InvalidColumnType("Option<String>")),
        }
    }
}

impl FromValue for i64 {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Integer(value) => Ok(*value),
            _ => Err(Error::InvalidColumnType("i64")),
        }
    }
}

impl FromValue for Vec<u8> {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Blob(value) => Ok(value.clone()),
            _ => Err(Error::InvalidColumnType("Vec<u8>")),
        }
    }
}

impl<'stmt> Row<'stmt> {
    pub fn get<I, T>(&self, index: I) -> Result<T>
    where
        I: RowIndex,
        T: FromValue,
    {
        let index = index.as_usize();
        let value = self
            .values
            .get(index)
            .ok_or(Error::InvalidColumnIndex(index))?;
        T::from_value(value)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct PersistedDb {
    snapshots: BTreeMap<String, PersistedSnapshotRow>,
    room_leases: BTreeMap<String, PersistedRoomLeaseRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSnapshotRow {
    doc_id: String,
    title: String,
    created_at: String,
    updated_at: String,
    access_token: String,
    update_bytes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedRoomLeaseRow {
    doc_id: String,
    node_id: String,
    base_url: Option<String>,
    lease_id: String,
    epoch: i64,
    activated_at: String,
    renewed_at: String,
    expires_at: String,
}

fn normalize_sql(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn load_db(path: &Path) -> Result<PersistedDb> {
    match fs::read(path) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(PersistedDb::default()),
        Err(error) => Err(Error::Io(error)),
    }
}

fn persist_db(path: &Path, db: &PersistedDb) -> Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }

    let temp_path = temp_data_path(path);
    let bytes = serde_json::to_vec(db)?;
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    replace_data_file(&temp_path, path)?;
    sync_parent_directory(path)?;
    Ok(())
}

fn replace_data_file(temp_path: &Path, path: &Path) -> Result<()> {
    match fs::rename(temp_path, path) {
        Ok(()) => Ok(()),
        #[cfg(windows)]
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::AlreadyExists | io::ErrorKind::PermissionDenied
            ) =>
        {
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(remove_error) if remove_error.kind() == io::ErrorKind::NotFound => {}
                Err(remove_error) => return Err(Error::Io(remove_error)),
            }
            fs::rename(temp_path, path)?;
            Ok(())
        }
        Err(error) => Err(Error::Io(error)),
    }
}

#[cfg(not(windows))]
fn sync_parent_directory(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        let directory = File::open(parent)?;
        directory.sync_all()?;
    }
    Ok(())
}

#[cfg(windows)]
fn sync_parent_directory(_path: &Path) -> Result<()> {
    Ok(())
}

fn query_rows(db: &PersistedDb, sql: &str, params: Vec<Value>) -> Result<Vec<Vec<Value>>> {
    let normalized = normalize_sql(sql);

    if normalized.starts_with(
        "SELECT doc_id, title, created_at, updated_at, access_token, update_bytes FROM snapshots WHERE doc_id = ?1",
    ) {
        let doc_id = expect_text_param(&params, 0, "?1")?;
        let Some(row) = db.snapshots.get(&doc_id) else {
            return Ok(Vec::new());
        };
        return Ok(vec![snapshot_values(row)?]);
    }

    if normalized.starts_with(
        "SELECT doc_id, title, created_at, updated_at, access_token, update_bytes FROM snapshots ORDER BY created_at ASC, doc_id ASC",
    ) {
        let mut rows = db.snapshots.values().cloned().collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.doc_id.cmp(&right.doc_id))
        });
        return rows
            .into_iter()
            .map(|row| snapshot_values(&row))
            .collect::<Result<Vec<_>>>();
    }

    if normalized.starts_with(
        "SELECT doc_id, node_id, base_url, lease_id, epoch, activated_at, renewed_at, expires_at FROM room_leases WHERE doc_id = ?1",
    ) {
        let doc_id = expect_text_param(&params, 0, "?1")?;
        let Some(row) = db.room_leases.get(&doc_id) else {
            return Ok(Vec::new());
        };
        return Ok(vec![room_lease_values(row)]);
    }

    if normalized.starts_with("SELECT COUNT(*) FROM room_leases WHERE doc_id = ?1") {
        let doc_id = expect_text_param(&params, 0, "?1")?;
        let count = if db.room_leases.contains_key(&doc_id) {
            1_i64
        } else {
            0_i64
        };
        return Ok(vec![vec![Value::Integer(count)]]);
    }

    Err(Error::InvalidQuery(normalized))
}

fn execute_mutation(db: &mut PersistedDb, sql: &str, params: Vec<Value>) -> Result<usize> {
    let normalized = normalize_sql(sql);

    if normalized.starts_with("INSERT INTO snapshots (") {
        let row = PersistedSnapshotRow {
            doc_id: expect_text_param(&params, 0, "?1")?,
            title: expect_text_param(&params, 1, "?2")?,
            created_at: expect_text_param(&params, 2, "?3")?,
            updated_at: expect_text_param(&params, 3, "?4")?,
            access_token: expect_text_param(&params, 4, "?5")?,
            update_bytes: encode_bytes(expect_blob_param(&params, 5, "?6")?.as_slice()),
        };
        db.snapshots.insert(row.doc_id.clone(), row);
        return Ok(1);
    }

    if normalized == "DELETE FROM snapshots WHERE doc_id = ?1" {
        let doc_id = expect_text_param(&params, 0, "?1")?;
        return Ok(usize::from(db.snapshots.remove(&doc_id).is_some()));
    }

    if normalized.starts_with("INSERT INTO room_leases (") {
        let uses_literal_null_base_url = normalized.contains("VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)");
        let row = PersistedRoomLeaseRow {
            doc_id: expect_text_param(&params, 0, "?1")?,
            node_id: expect_text_param(&params, 1, "?2")?,
            base_url: if uses_literal_null_base_url {
                None
            } else {
                expect_optional_text_param(&params, 2, "?3")?
            },
            lease_id: expect_text_param(
                &params,
                if uses_literal_null_base_url { 2 } else { 3 },
                if uses_literal_null_base_url { "?3" } else { "?4" },
            )?,
            epoch: expect_i64_param(
                &params,
                if uses_literal_null_base_url { 3 } else { 4 },
                if uses_literal_null_base_url { "?4" } else { "?5" },
            )?,
            activated_at: expect_text_param(
                &params,
                if uses_literal_null_base_url { 4 } else { 5 },
                if uses_literal_null_base_url { "?5" } else { "?6" },
            )?,
            renewed_at: expect_text_param(
                &params,
                if uses_literal_null_base_url { 5 } else { 6 },
                if uses_literal_null_base_url { "?6" } else { "?7" },
            )?,
            expires_at: expect_text_param(
                &params,
                if uses_literal_null_base_url { 6 } else { 7 },
                if uses_literal_null_base_url { "?7" } else { "?8" },
            )?,
        };
        db.room_leases.insert(row.doc_id.clone(), row);
        return Ok(1);
    }

    if normalized.starts_with("UPDATE room_leases SET base_url = ?2, renewed_at = ?3, expires_at = ?4 WHERE doc_id = ?1 AND node_id = ?5 AND lease_id = ?6 AND epoch = ?7")
    {
        let doc_id = expect_text_param(&params, 0, "?1")?;
        let Some(row) = db.room_leases.get_mut(&doc_id) else {
            return Ok(0);
        };
        let node_id = expect_text_param(&params, 4, "?5")?;
        let lease_id = expect_text_param(&params, 5, "?6")?;
        let epoch = expect_i64_param(&params, 6, "?7")?;
        if row.node_id != node_id || row.lease_id != lease_id || row.epoch != epoch {
            return Ok(0);
        }
        row.base_url = expect_optional_text_param(&params, 1, "?2")?;
        row.renewed_at = expect_text_param(&params, 2, "?3")?;
        row.expires_at = expect_text_param(&params, 3, "?4")?;
        return Ok(1);
    }

    if normalized.starts_with(
        "DELETE FROM room_leases WHERE doc_id = ?1 AND node_id = ?2 AND lease_id = ?3 AND epoch = ?4",
    ) {
        let doc_id = expect_text_param(&params, 0, "?1")?;
        let node_id = expect_text_param(&params, 1, "?2")?;
        let lease_id = expect_text_param(&params, 2, "?3")?;
        let epoch = expect_i64_param(&params, 3, "?4")?;
        let matches = db.room_leases.get(&doc_id).is_some_and(|row| {
            row.node_id == node_id && row.lease_id == lease_id && row.epoch == epoch
        });
        if matches {
            db.room_leases.remove(&doc_id);
            return Ok(1);
        }
        return Ok(0);
    }

    Err(Error::InvalidQuery(normalized))
}

fn snapshot_values(row: &PersistedSnapshotRow) -> Result<Vec<Value>> {
    Ok(vec![
        Value::Text(row.doc_id.clone()),
        Value::Text(row.title.clone()),
        Value::Text(row.created_at.clone()),
        Value::Text(row.updated_at.clone()),
        Value::Text(row.access_token.clone()),
        Value::Blob(decode_bytes(&row.update_bytes)?),
    ])
}

fn room_lease_values(row: &PersistedRoomLeaseRow) -> Vec<Value> {
    vec![
        Value::Text(row.doc_id.clone()),
        Value::Text(row.node_id.clone()),
        match &row.base_url {
            Some(value) => Value::Text(value.clone()),
            None => Value::Null,
        },
        Value::Text(row.lease_id.clone()),
        Value::Integer(row.epoch),
        Value::Text(row.activated_at.clone()),
        Value::Text(row.renewed_at.clone()),
        Value::Text(row.expires_at.clone()),
    ]
}

fn expect_text_param(params: &[Value], index: usize, placeholder: &str) -> Result<String> {
    match params.get(index) {
        Some(Value::Text(value)) => Ok(value.clone()),
        Some(_) => Err(Error::InvalidParameter(format!(
            "expected text parameter `{placeholder}`"
        ))),
        None => Err(Error::InvalidParameter(format!(
            "missing parameter `{placeholder}`"
        ))),
    }
}

fn expect_optional_text_param(
    params: &[Value],
    index: usize,
    placeholder: &str,
) -> Result<Option<String>> {
    match params.get(index) {
        Some(Value::Null) => Ok(None),
        Some(Value::Text(value)) => Ok(Some(value.clone())),
        Some(_) => Err(Error::InvalidParameter(format!(
            "expected nullable text parameter `{placeholder}`"
        ))),
        None => Err(Error::InvalidParameter(format!(
            "missing parameter `{placeholder}`"
        ))),
    }
}

fn expect_i64_param(params: &[Value], index: usize, placeholder: &str) -> Result<i64> {
    match params.get(index) {
        Some(Value::Integer(value)) => Ok(*value),
        Some(_) => Err(Error::InvalidParameter(format!(
            "expected integer parameter `{placeholder}`"
        ))),
        None => Err(Error::InvalidParameter(format!(
            "missing parameter `{placeholder}`"
        ))),
    }
}

fn expect_blob_param(params: &[Value], index: usize, placeholder: &str) -> Result<Vec<u8>> {
    match params.get(index) {
        Some(Value::Blob(value)) => Ok(value.clone()),
        Some(_) => Err(Error::InvalidParameter(format!(
            "expected blob parameter `{placeholder}`"
        ))),
        None => Err(Error::InvalidParameter(format!(
            "missing parameter `{placeholder}`"
        ))),
    }
}

fn encode_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(nibble_to_hex(byte >> 4));
        encoded.push(nibble_to_hex(byte & 0x0f));
    }
    encoded
}

fn decode_bytes(encoded: &str) -> Result<Vec<u8>> {
    if encoded.len() % 2 != 0 {
        return Err(Error::InvalidParameter(
            "sqlite shim hex payload has odd length".to_owned(),
        ));
    }

    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = hex_to_nibble(pair[0])?;
        let low = hex_to_nibble(pair[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + (nibble - 10)) as char,
        _ => unreachable!("nibble is always <= 0x0f"),
    }
}

fn hex_to_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(Error::InvalidParameter(
            "sqlite shim hex payload contains an invalid character".to_owned(),
        )),
    }
}

struct FileLock {
    path: PathBuf,
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn acquire_file_lock(path: &Path, timeout: Duration) -> Result<FileLock> {
    let lock_path = lock_path(path);
    let start = Instant::now();
    loop {
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                file.write_all(b"backend-rusqlite-shim-lock")?;
                file.sync_all()?;
                return Ok(FileLock { path: lock_path });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                if start.elapsed() >= timeout {
                    return Err(Error::Io(io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!(
                            "timed out waiting for sqlite shim lock `{}`",
                            lock_path.display()
                        ),
                    )));
                }
                thread::sleep(LOCK_RETRY_INTERVAL);
            }
            Err(error) => return Err(Error::Io(error)),
        }
    }
}

fn lock_path(path: &Path) -> PathBuf {
    let suffix = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| format!("{file_name}.lock"))
        .unwrap_or_else(|| "sqlite-shim.lock".to_owned());
    path.with_file_name(suffix)
}

fn temp_data_path(path: &Path) -> PathBuf {
    let suffix = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| format!("{file_name}.tmp"))
        .unwrap_or_else(|| "sqlite-shim.tmp".to_owned());
    path.with_file_name(suffix)
}
