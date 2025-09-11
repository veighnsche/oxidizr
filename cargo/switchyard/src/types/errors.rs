use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("invalid path")]
    InvalidPath,
    #[error("io error")]
    Io,
    #[error("policy violation")]
    Policy,
}

#[derive(Debug, Error)]
#[error("{kind:?}: {msg}")]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: String,
}

pub type Result<T> = std::result::Result<T, Error>;
