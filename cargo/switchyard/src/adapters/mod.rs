pub mod attest;
pub mod lock; // contains mod.rs and file.rs
pub mod ownership; // contains mod.rs and fs.rs
pub mod path;
pub mod smoke;
// Compatibility shim for old path switchyard::adapters::lock_file::FileLockManager
#[deprecated(note = "Deprecated shim: use `switchyard::adapters::lock::file::*` instead. This `lock_file` module will be removed in 0.2.")]
pub mod lock_file {
    pub use super::lock::file::*;
}

pub use attest::*;
pub use lock::file::FileLockManager;
pub use lock::*;
pub use ownership::fs::FsOwnershipOracle;
pub use ownership::*;
pub use path::*;
pub use smoke::*;
