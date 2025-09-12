pub mod atomic;
pub mod backup;
pub mod restore;
pub mod swap;
pub mod paths;
pub mod meta;
pub mod mount;

pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};
pub use backup::{backup_path_with_tag, has_backup_artifacts, create_snapshot};
pub use paths::is_safe_path;
pub use restore::{restore_file, restore_file_prev};
pub use swap::replace_file_with_symlink;
pub use meta::{kind_of, resolve_symlink_target, detect_preservation_capabilities, sha256_hex_of};
pub use mount::{ensure_rw_exec, ProcStatfsInspector};
