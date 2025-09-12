pub mod attest;
pub mod lock;        // contains mod.rs and file.rs
pub mod ownership;   // contains mod.rs and fs.rs
pub mod smoke;
pub mod path;
// Compatibility shim for old path switchyard::adapters::lock_file::FileLockManager
pub mod lock_file { pub use super::lock::file::*; }

pub use attest::*;
pub use lock::*;
pub use lock::file::FileLockManager;
pub use ownership::*;
pub use ownership::fs::FsOwnershipOracle;
pub use smoke::*;
pub use path::*;
