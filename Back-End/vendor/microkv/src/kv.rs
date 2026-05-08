#![allow(clippy::result_map_unit_fn)]

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::errors::{ErrorType, KVError, Result};
use crate::namespace::NamespaceMicrokv;

const DEFAULT_WORKSPACE_PATH: &str = ".microkv/";

type KV = IndexMap<String, Vec<u8>>;

#[derive(Clone, Serialize, Deserialize)]
pub struct MicroKV {
    path: PathBuf,
    storage: Arc<RwLock<KV>>,
    nonce: [u8; 24],
    #[serde(skip_serializing, skip_deserializing)]
    pwd: Option<Vec<u8>>,
    is_auto_commit: bool,
}

impl MicroKV {
    pub fn new_with_base_path<S: AsRef<str>>(dbname: S, base_path: PathBuf) -> Self {
        let path = Self::get_db_path_with_base_path(dbname, base_path);
        Self {
            path,
            storage: Arc::new(RwLock::new(KV::new())),
            nonce: [0; 24],
            pwd: None,
            is_auto_commit: false,
        }
    }

    pub fn new<S: AsRef<str>>(dbname: S) -> Self {
        let mut path = Self::get_home_dir();
        path.push(DEFAULT_WORKSPACE_PATH);
        Self::new_with_base_path(dbname, path)
    }

    pub fn open_with_base_path<S: AsRef<str>>(dbname: S, base_path: PathBuf) -> Result<Self> {
        let path = Self::get_db_path_with_base_path(dbname.as_ref(), base_path.clone());

        if path.is_file() {
            let mut kv_raw = Vec::new();
            File::open(path)?.read_to_end(&mut kv_raw)?;
            let mut kv: Self = bincode::deserialize(&kv_raw).map_err(|error| KVError {
                error: ErrorType::KVError,
                msg: Some(format!("cannot deserialize persisted microkv state: {error}")),
            })?;
            kv.path = Self::get_db_path_with_base_path(dbname, base_path);
            Ok(kv)
        } else {
            Ok(Self::new_with_base_path(dbname, base_path))
        }
    }

    pub fn open<S: AsRef<str>>(dbname: S) -> Result<Self> {
        let mut path = Self::get_home_dir();
        path.push(DEFAULT_WORKSPACE_PATH);
        Self::open_with_base_path(dbname, path)
    }

    fn get_home_dir() -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn get_db_path<S: AsRef<str>>(name: S) -> PathBuf {
        let mut path = Self::get_home_dir();
        path.push(DEFAULT_WORKSPACE_PATH);
        Self::get_db_path_with_base_path(name, path)
    }

    pub fn get_db_path_with_base_path<S: AsRef<str>>(name: S, mut base_path: PathBuf) -> PathBuf {
        base_path.push(name.as_ref());
        base_path.set_extension("kv");
        base_path
    }

    pub fn with_pwd_clear<S: AsRef<str>>(mut self, unsafe_pwd: S) -> Self {
        self.pwd = Some(unsafe_pwd.as_ref().as_bytes().to_vec());
        self
    }

    pub fn with_pwd_hash(mut self, pwd: [u8; 32]) -> Self {
        self.pwd = Some(pwd.to_vec());
        self
    }

    pub fn set_auto_commit(mut self, enable: bool) -> Self {
        self.is_auto_commit = enable;
        self
    }

    pub fn namespace(&self, namespace: impl AsRef<str>) -> NamespaceMicrokv<'_> {
        NamespaceMicrokv::new(namespace, self)
    }

    pub fn namespace_default(&self) -> NamespaceMicrokv<'_> {
        self.namespace("")
    }

    pub fn get_unwrap<V>(&self, key: impl AsRef<str>) -> Result<V>
    where
        V: DeserializeOwned + 'static,
    {
        self.namespace_default().get_unwrap(key)
    }

    pub fn get<V>(&self, key: impl AsRef<str>) -> Result<Option<V>>
    where
        V: DeserializeOwned + 'static,
    {
        self.namespace_default().get(key)
    }

    pub fn put<V>(&self, key: impl AsRef<str>, value: &V) -> Result<()>
    where
        V: Serialize,
    {
        self.namespace_default().put(key, value)
    }

    pub fn delete(&self, key: impl AsRef<str>) -> Result<()> {
        self.namespace_default().delete(key)
    }

    pub fn exists(&self, key: impl AsRef<str>) -> Result<bool> {
        self.namespace_default().exists(key)
    }

    pub fn keys(&self) -> Result<Vec<String>> {
        self.namespace_default().keys()
    }

    pub fn sorted_keys(&self) -> Result<Vec<String>> {
        self.namespace_default().sorted_keys()
    }

    pub fn clear(&self) -> Result<()> {
        self.namespace_default().clear()
    }

    pub fn lock_read<C, R>(&self, callback: C) -> Result<R>
    where
        C: FnOnce(&KV) -> R,
    {
        let data = self.storage.read().map_err(|_| KVError {
            error: ErrorType::PoisonError,
            msg: None,
        })?;
        Ok(callback(&data))
    }

    pub fn lock_write<C, R>(&self, callback: C) -> Result<R>
    where
        C: FnOnce(&mut KV) -> R,
    {
        let mut data = self.storage.write().map_err(|_| KVError {
            error: ErrorType::PoisonError,
            msg: None,
        })?;
        let result = callback(&mut data);
        drop(data);

        if self.is_auto_commit {
            self.commit()?;
        }

        Ok(result)
    }

    pub fn commit(&self) -> Result<()> {
        match self.path.parent() {
            Some(path) => {
                if !path.is_dir() {
                    fs::create_dir_all(path)?;
                }
            }
            None => {
                return Err(KVError {
                    error: ErrorType::FileError,
                    msg: Some("The store file parent path isn't sound".to_owned()),
                });
            }
        }

        let path = Path::new(&self.path);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let payload = bincode::serialize(self).map_err(|error| KVError {
            error: ErrorType::KVError,
            msg: Some(format!("cannot serialize microkv state: {error}")),
        })?;
        file.write_all(&payload)?;
        Ok(())
    }

    pub fn destruct(&self) -> Result<()> {
        unimplemented!()
    }
}

impl Drop for MicroKV {
    fn drop(&mut self) {
        if let Some(pwd) = &mut self.pwd {
            pwd.fill(0);
        }
    }
}
