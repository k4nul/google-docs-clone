use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Borrow;

use crate::errors::{ErrorType, KVError, Result};
use crate::MicroKV;

#[derive(Clone)]
pub struct NamespaceMicrokv<'a> {
    namespace: String,
    microkv: &'a MicroKV,
}

pub fn format_key(namespace: &str, key: impl AsRef<str>) -> String {
    if namespace.is_empty() {
        key.as_ref().to_string()
    } else {
        format!("{}@{}", namespace, key.as_ref())
    }
}

impl<'a> NamespaceMicrokv<'a> {
    pub fn new(namespace: impl AsRef<str>, microkv: &'a MicroKV) -> Self {
        Self {
            namespace: namespace.as_ref().to_string(),
            microkv,
        }
    }

    pub fn get_unwrap<V>(&self, key: impl AsRef<str>) -> Result<V>
    where
        V: DeserializeOwned + 'static,
    {
        self.microkv
            .lock_read(|c| c.kv_get_unwrap(self.microkv, &self.namespace, &key))?
    }

    pub fn get<V>(&self, key: impl AsRef<str>) -> Result<Option<V>>
    where
        V: DeserializeOwned + 'static,
    {
        self.microkv
            .lock_read(|c| c.kv_get(self.microkv, &self.namespace, &key))?
    }

    pub fn put<V>(&self, key: impl AsRef<str>, value: &V) -> Result<()>
    where
        V: Serialize,
    {
        self.microkv
            .lock_write(|c| c.kv_put(self.microkv, &self.namespace, &key, value))
    }

    pub fn delete(&self, key: impl AsRef<str>) -> Result<()> {
        self.microkv
            .lock_write(|c| c.kv_delete(&self.namespace, &key))
    }

    pub fn exists(&self, key: impl AsRef<str>) -> Result<bool> {
        self.microkv
            .lock_read(|c| c.kv_exists(&self.namespace, &key))
    }

    pub fn keys(&self) -> Result<Vec<String>> {
        self.microkv.lock_read(|c| {
            c.keys()
                .filter(|x| {
                    if self.namespace.is_empty() {
                        return true;
                    }
                    x.starts_with(&format_key(&self.namespace, ""))
                })
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
        })
    }

    pub fn sorted_keys(&self) -> Result<Vec<String>> {
        self.microkv.lock_write(|c| {
            c.sort_keys();
            c.keys()
                .filter(|x| {
                    if self.namespace.is_empty() {
                        return true;
                    }
                    x.starts_with(&format_key(&self.namespace, ""))
                })
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
        })
    }

    pub fn clear(&self) -> Result<()> {
        self.microkv.lock_write(|c| {
            if self.namespace.is_empty() {
                c.clear();
            } else {
                c.retain(|key, _| !key.starts_with(&format_key(&self.namespace, "")));
            }
        })
    }
}

pub trait ExtendedIndexMap {
    fn kv_delete(&mut self, namespace: impl AsRef<str>, key: impl AsRef<str>);

    fn kv_exists(&self, namespace: impl AsRef<str>, key: impl AsRef<str>) -> bool;

    fn kv_get<V>(
        &self,
        microkv: &MicroKV,
        namespace: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<Option<V>>
    where
        V: DeserializeOwned + 'static;

    fn kv_get_unwrap<V>(
        &self,
        microkv: &MicroKV,
        namespace: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<V>
    where
        V: DeserializeOwned + 'static;

    fn kv_put<V>(
        &mut self,
        microkv: &MicroKV,
        namespace: impl AsRef<str>,
        key: impl AsRef<str>,
        value: &V,
    ) where
        V: Serialize;
}

impl ExtendedIndexMap for IndexMap<String, Vec<u8>> {
    fn kv_delete(&mut self, namespace: impl AsRef<str>, key: impl AsRef<str>) {
        self.remove(&format_key(namespace.as_ref(), key));
    }

    fn kv_exists(&self, namespace: impl AsRef<str>, key: impl AsRef<str>) -> bool {
        self.contains_key(&format_key(namespace.as_ref(), key))
    }

    fn kv_get<V>(
        &self,
        _microkv: &MicroKV,
        namespace: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<Option<V>>
    where
        V: DeserializeOwned + 'static,
    {
        parse_value(self.get(&format_key(namespace.as_ref(), key)))
    }

    fn kv_get_unwrap<V>(
        &self,
        microkv: &MicroKV,
        namespace: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<V>
    where
        V: DeserializeOwned + 'static,
    {
        if let Some(value) = self.kv_get(microkv, namespace, key)? {
            Ok(value)
        } else {
            Err(KVError {
                error: ErrorType::KVError,
                msg: Some("key not found in storage".to_owned()),
            })
        }
    }

    fn kv_put<V>(
        &mut self,
        _microkv: &MicroKV,
        namespace: impl AsRef<str>,
        key: impl AsRef<str>,
        value: &V,
    ) where
        V: Serialize,
    {
        let data_key = format_key(namespace.as_ref(), key);
        let payload = bincode::serialize(value).unwrap();
        self.insert(data_key, payload);
    }
}

fn parse_value<T, V>(value: Option<T>) -> Result<Option<V>>
where
    T: Borrow<Vec<u8>>,
    V: DeserializeOwned + 'static,
{
    match value {
        Some(bytes) => {
            let value = bincode::deserialize(bytes.borrow()).map_err(|_| KVError {
                error: ErrorType::KVError,
                msg: Some("cannot deserialize into specified object type".to_owned()),
            })?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}
