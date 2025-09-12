pub mod atomic;
pub mod backup;
pub mod meta;
pub mod mount;
pub mod paths;
pub mod restore;
pub mod swap;

pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};
pub use backup::{backup_path_with_tag, create_snapshot, has_backup_artifacts};
pub use meta::{detect_preservation_capabilities, kind_of, resolve_symlink_target, sha256_hex_of};
pub use mount::{ensure_rw_exec, ProcStatfsInspector};
pub use paths::is_safe_path;
pub use restore::{restore_file, restore_file_prev};
pub use swap::replace_file_with_symlink;
