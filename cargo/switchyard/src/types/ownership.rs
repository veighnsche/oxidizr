//! Data-only type for ownership information of a filesystem path.
//! Centralized under `crate::types` for cross-layer reuse.

#[derive(Clone, Debug)]
pub struct OwnershipInfo {
    pub uid: u32,
    pub gid: u32,
    pub pkg: String,
}
