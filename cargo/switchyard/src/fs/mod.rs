//! Filesystem primitives used by Switchyard.
//!
//! This module provides low-level, TOCTOU-safe filesystem operations used by the
//! higher-level API stages. Consumers should prefer calling public API stages in
//! `switchyard::api` rather than these atoms. Some re-exports below are kept for
//! backwards compatibility and are deprecated.

pub mod atomic;
pub mod backup;
pub mod meta;
pub mod mount;
pub mod paths;
pub mod restore;
pub mod swap;

#[deprecated(
    note = "Low-level FS atoms are internal: prefer high-level API. This re-export will be removed in 0.2."
)]
pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};
pub use backup::{backup_path_with_tag, create_snapshot, has_backup_artifacts};
pub use meta::{detect_preservation_capabilities, kind_of, resolve_symlink_target, sha256_hex_of};
pub use mount::{ensure_rw_exec, ProcStatfsInspector};
pub use paths::is_safe_path;
pub use restore::{restore_file, restore_file_prev};
pub use swap::replace_file_with_symlink;
