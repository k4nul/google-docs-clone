use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt,
    ops::Bound,
};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub mod mem_store {
    use super::MemKvStore;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct MemKvConfig;

    impl MemKvConfig {
        pub fn new() -> Self {
            Self
        }

        pub fn build(self) -> MemKvStore {
            MemKvStore::default()
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Serde(serde_json::Error),
    InvalidHex(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serde(error) => write!(f, "{error}"),
            Self::InvalidHex(value) => write!(f, "invalid hex payload `{value}`"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Serde(error) => Some(error),
            Self::InvalidHex(_) => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemKvStore {
    entries: BTreeMap<Vec<u8>, Bytes>,
}

impl MemKvStore {
    pub fn import_all(&mut self, bytes: Bytes) -> Result<(), Error> {
        let stored = serde_json::from_slice::<StoredEntries>(&bytes)?;
        let mut entries = BTreeMap::new();
        for (key, value) in stored.entries {
            entries.insert(decode_bytes(&key)?, Bytes::from(decode_bytes(&value)?));
        }
        self.entries = entries;
        Ok(())
    }

    pub fn export_all(&self) -> Bytes {
        let stored = StoredEntries {
            entries: self
                .entries
                .iter()
                .map(|(key, value)| (encode_bytes(key), encode_bytes(value.as_ref())))
                .collect(),
        };

        Bytes::from(
            serde_json::to_vec(&stored).expect("MemKvStore export serialization should not fail"),
        )
    }

    pub fn get(&self, key: &[u8]) -> Option<Bytes> {
        self.entries.get(key).cloned()
    }

    pub fn set(&mut self, key: &[u8], value: Bytes) {
        self.entries.insert(key.to_vec(), value);
    }

    pub fn remove(&mut self, key: &[u8]) {
        self.entries.remove(key);
    }

    pub fn scan(
        &self,
        start: Bound<Vec<u8>>,
        end: Bound<Vec<u8>>,
    ) -> impl Iterator<Item = (Bytes, Bytes)> {
        self.entries
            .range((start, end))
            .map(|(key, value)| (Bytes::copy_from_slice(key), value.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct StoredEntries {
    entries: BTreeMap<String, String>,
}

fn encode_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_bytes(value: &str) -> Result<Vec<u8>, Error> {
    if value.len() % 2 != 0 {
        return Err(Error::InvalidHex(value.to_owned()));
    }

    let mut decoded = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let high = decode_nibble(bytes[index]).ok_or_else(|| Error::InvalidHex(value.to_owned()))?;
        let low =
            decode_nibble(bytes[index + 1]).ok_or_else(|| Error::InvalidHex(value.to_owned()))?;
        decoded.push((high << 4) | low);
        index += 2;
    }

    Ok(decoded)
}

fn decode_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
