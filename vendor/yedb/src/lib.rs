use std::{
    error::Error as StdError,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const KEYS_DIR_NAME: &str = "keys";
const META_FILE_NAME: &str = ".yedb";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ErrorKind {
    IOError,
    DataError,
    TimeoutError,
    KeyNotFound,
    Busy,
    NotOpened,
    NotInitialized,
    InvalidParameter,
    #[default]
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::IOError => "I/O Error",
            Self::DataError => "Data error",
            Self::TimeoutError => "Timeout error",
            Self::KeyNotFound => "Key not found",
            Self::Busy => "Database is busy",
            Self::NotOpened => "Not opened",
            Self::NotInitialized => "Not initialized",
            Self::InvalidParameter => "Invalid parameter",
            Self::Other => "Error",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    error_kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, error: impl fmt::Display) -> Self {
        Self {
            error_kind: kind,
            message: error.to_string(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.error_kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerInfo;

#[derive(Debug, Clone)]
pub struct Database {
    path: String,
    key_path: String,
    meta_path: String,
    opened: bool,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            path: String::new(),
            key_path: String::new(),
            meta_path: String::new(),
            opened: false,
        }
    }

    pub fn set_db_path(&mut self, path: &str) -> Result<(), Error> {
        if self.opened {
            return Err(Error::new(ErrorKind::Busy, "the database is opened"));
        }
        if path.trim().is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidParameter,
                "database path cannot be empty",
            ));
        }

        self.path = path.to_owned();
        self.key_path = format!("{}/{}", self.path, KEYS_DIR_NAME);
        self.meta_path = format!("{}/{}", self.path, META_FILE_NAME);
        Ok(())
    }

    pub fn open(&mut self) -> Result<ServerInfo, Error> {
        if self.path.is_empty() {
            return Err(Error::new(ErrorKind::NotInitialized, "db path not set"));
        }
        if self.opened {
            return Err(Error::new(
                ErrorKind::Busy,
                "the database is already opened",
            ));
        }

        fs::create_dir_all(&self.key_path).map_err(io_error)?;
        if !Path::new(&self.meta_path).exists() {
            let mut meta_file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&self.meta_path)
                .map_err(io_error)?;
            meta_file
                .write_all(br#"{"engine":"shim"}"#)
                .map_err(io_error)?;
            meta_file.sync_all().map_err(io_error)?;
        }
        sync_dir(Path::new(&self.path)).map_err(io_error)?;
        self.opened = true;
        Ok(ServerInfo)
    }

    pub fn key_get(&mut self, key: &str) -> Result<Value, Error> {
        let path = self.key_file_path(key)?;
        let bytes = fs::read(&path).map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => Error::new(ErrorKind::KeyNotFound, format!("missing key `{key}`")),
            _ => io_error(error),
        })?;
        serde_json::from_slice(&bytes)
            .map_err(|error| Error::new(ErrorKind::DataError, format!("{key}: {error}")))
    }

    pub fn key_set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        let path = self.key_file_path(key)?;
        let parent = path.parent().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidParameter,
                format!("invalid key `{key}`"),
            )
        })?;
        fs::create_dir_all(parent).map_err(io_error)?;

        let temp_path = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec(&value)
            .map_err(|error| Error::new(ErrorKind::DataError, format!("{key}: {error}")))?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)
            .map_err(io_error)?;
        file.write_all(&bytes).map_err(io_error)?;
        file.sync_all().map_err(io_error)?;
        drop(file);

        fs::rename(&temp_path, &path).map_err(io_error)?;
        sync_dir(parent).map_err(io_error)?;
        Ok(())
    }

    pub fn key_delete(&mut self, key: &str) -> Result<(), Error> {
        let path = self.key_file_path(key)?;
        match fs::remove_file(&path) {
            Ok(()) => {
                if let Some(parent) = path.parent() {
                    let _ = cleanup_empty_parents(parent, Path::new(&self.key_path));
                    let _ = sync_dir(parent);
                }
                Ok(())
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Err(Error::new(ErrorKind::KeyNotFound, format!("missing key `{key}`")))
            }
            Err(error) => Err(io_error(error)),
        }
    }

    pub fn key_list_all(&mut self, key: &str) -> Result<Vec<String>, Error> {
        self.ensure_opened()?;
        let relative_parts = normalize_key_segments(key)?;
        let mut root = PathBuf::from(&self.key_path);
        for part in &relative_parts {
            root.push(part);
        }
        if !root.exists() {
            return Ok(Vec::new());
        }

        let mut keys = Vec::new();
        collect_keys(&root, &relative_parts, &mut keys).map_err(io_error)?;
        keys.sort();
        Ok(keys)
    }

    fn ensure_opened(&self) -> Result<(), Error> {
        if self.opened {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotOpened, "database is not opened"))
        }
    }

    fn key_file_path(&self, key: &str) -> Result<PathBuf, Error> {
        self.ensure_opened()?;
        let parts = normalize_key_segments(key)?;
        let mut path = PathBuf::from(&self.key_path);
        for part in &parts {
            path.push(part);
        }
        path.set_extension("json");
        Ok(path)
    }
}

fn normalize_key_segments(key: &str) -> Result<Vec<&str>, Error> {
    let parts = key
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty()
        || parts
            .iter()
            .any(|segment| matches!(*segment, "." | "..") || segment.contains('\\'))
    {
        return Err(Error::new(
            ErrorKind::InvalidParameter,
            format!("invalid key `{key}`"),
        ));
    }
    Ok(parts)
}

fn collect_keys(root: &Path, prefix: &[&str], keys: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy().into_owned();
            let mut next_prefix = prefix.iter().map(|segment| (*segment).to_owned()).collect::<Vec<_>>();
            next_prefix.push(name);
            let borrowed = next_prefix.iter().map(String::as_str).collect::<Vec<_>>();
            collect_keys(&path, &borrowed, keys)?;
            continue;
        }

        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let stem = path.file_stem().and_then(|value| value.to_str()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid yedb key filename")
        })?;
        let mut key = prefix.iter().map(|segment| (*segment).to_owned()).collect::<Vec<_>>();
        key.push(stem.to_owned());
        keys.push(key.join("/"));
    }
    Ok(())
}

fn cleanup_empty_parents(path: &Path, stop_at: &Path) -> io::Result<()> {
    let mut current = path.to_path_buf();
    while current.starts_with(stop_at) && current != stop_at {
        if fs::read_dir(&current)?.next().is_some() {
            break;
        }
        fs::remove_dir(&current)?;
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent.to_path_buf();
    }
    Ok(())
}

fn sync_dir(path: &Path) -> io::Result<()> {
    let file = File::open(path)?;
    file.sync_all()
}

fn io_error(error: io::Error) -> Error {
    Error::new(ErrorKind::IOError, error)
}
