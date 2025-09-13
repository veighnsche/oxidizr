//! Restore subsystem — split from monolith per `zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md`

pub mod engine;
pub mod idempotence;
pub mod integrity;
pub mod selector;
pub mod steps;
pub mod types;

pub use engine::{restore_file, restore_file_prev};
