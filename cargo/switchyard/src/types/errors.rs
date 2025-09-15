//! Error types used across Switchyard.
use thiserror::Error;

/// High-level error categories for type-level operations and adapters.
#[derive(Debug, Copy, Clone, Error)]
pub enum ErrorKind {
    #[error("invalid path")]
    InvalidPath,
    #[error("io error")]
    Io,
    #[error("policy violation")]
    Policy,
}

/// Structured error with a kind and human message.
#[derive(Debug, Error)]
#[error("{kind:?}: {msg}")]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: String,
}

/// Convenient alias for results returning a `types::Error`.
pub type Result<T> = std::result::Result<T, Error>;
