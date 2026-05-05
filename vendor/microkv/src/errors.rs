use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, KVError>;

#[derive(Debug)]
pub enum ErrorType {
    KVError,
    CryptoError,
    FileError,
    PoisonError,
}

pub struct KVError {
    pub error: ErrorType,
    pub msg: Option<String>,
}

impl fmt::Debug for KVError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = &self.msg {
            write!(f, "KVError::{:?}: {msg}", self.error)
        } else {
            write!(f, "KVError::{:?}", self.error)
        }
    }
}

impl fmt::Display for KVError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = &self.msg {
            write!(
                f,
                "{:?} received from microkv with message: {msg}",
                self.error
            )
        } else {
            write!(f, "{:?} received from microkv", self.error)
        }
    }
}

impl From<std::io::Error> for KVError {
    fn from(error: std::io::Error) -> Self {
        KVError {
            error: ErrorType::FileError,
            msg: Some(error.to_string()),
        }
    }
}

impl Error for KVError {}
