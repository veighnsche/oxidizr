//! Data-only type for ownership information of a filesystem path.
//! Centralized under `crate::types` for cross-layer reuse.

/// Typed representation of ownership information for a filesystem path.
/// Centralized under `crate::types` for cross-layer reuse.
#[derive(Clone, Debug)]
pub struct OwnershipInfo {
    /// User ID of the owner
    pub uid: u32,
    /// Group ID of the owner
    pub gid: u32,
    /// Package name associated with the ownership
    pub pkg: String,
}
