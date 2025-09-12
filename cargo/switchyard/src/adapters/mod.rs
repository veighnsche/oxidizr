pub mod attest;
pub mod lock; // contains mod.rs and file.rs
pub mod ownership; // contains mod.rs and fs.rs
pub mod path;
pub mod smoke;
// Compatibility shim for old path switchyard::adapters::lock_file::FileLockManager
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
