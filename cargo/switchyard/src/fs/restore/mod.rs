//! Restore subsystem — split from monolith per zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md

pub mod types;
pub mod selector;
pub mod idempotence;
pub mod integrity;
pub mod steps;
pub mod engine;

pub use engine::{restore_file, restore_file_prev};
