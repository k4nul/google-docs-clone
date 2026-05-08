//! Minimal vendored fork of `microkv` for this repository.
//!
//! The backend only relies on the unencrypted persistent KV surface, so this fork
//! preserves the repo-used API while removing the native `libsodium` dependency
//! that dominated cold build time.

pub mod errors;
pub mod kv;
pub mod namespace;

pub use self::kv::MicroKV;
