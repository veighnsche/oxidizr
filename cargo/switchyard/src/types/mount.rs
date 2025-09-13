//! Data-only mount types used across the crate.

/// Typed representation of mount flags.
/// Centralized under `crate::types` for cross-layer reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountFlags {
    /// Whether the mount is read-only
    pub read_only: bool,
    /// Whether the mount has execution disabled
    pub no_exec: bool,
}

/// Error types for mount operations.
/// Centralized under `crate::types` for cross-layer reuse.
#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum MountError {
    /// Unknown or ambiguous mount state
    #[error("unknown or ambiguous mount state")]
    Unknown,
}
