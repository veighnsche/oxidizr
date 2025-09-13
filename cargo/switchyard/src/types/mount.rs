//! Data-only mount types used across the crate.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountFlags {
    pub read_only: bool,
    pub no_exec: bool,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum MountError {
    #[error("unknown or ambiguous mount state")]
    Unknown,
}
