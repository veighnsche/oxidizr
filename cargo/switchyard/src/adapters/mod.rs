pub mod attest;
pub mod lock; // contains mod.rs and file.rs
pub mod ownership; // contains mod.rs and fs.rs
pub mod path;
pub mod smoke;
/// BEGIN REMOVE BLOCK — deprecated shim: use switchyard::adapters::lock::file::*
/// removed in refactor sweep
/// END REMOVE BLOCK

pub use attest::*;
pub use lock::file::FileLockManager;
pub use lock::*;
pub use ownership::fs::FsOwnershipOracle;
pub use ownership::*;
pub use path::*;
pub use smoke::*;
