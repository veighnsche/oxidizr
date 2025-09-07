//! A scaffolding library for safely switching system core utilities
//! (oxidizr-arch inspired). This is intentionally minimal and non-destructive.

pub mod cli;
pub mod config;
pub mod error;
pub mod experiments;
pub mod package_manager;
pub mod utils;

// Re-export commonly used items
pub use crate::error::{CoreutilsError, Result};

// Backward-compatibility re-exports for legacy module paths
pub use crate::experiments as experiment;
pub use crate::utils::worker;

#[cfg(test)]
mod tests {
    use crate::experiments::UutilsExperiment;
    use crate::utils::test_utils::MockWorker;
    use std::path::PathBuf;

    #[test]
    fn test_check_compatible_scaffold() {
        let exp = UutilsExperiment {
            name: "coreutils".into(),
            package_name: "uutils-coreutils".into(),
            unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
            bin_directory: PathBuf::from("/usr/lib/uutils/coreutils"),
        };
        let worker = MockWorker::default();
        let ok = exp.check_compatible(&worker).unwrap();
        assert!(ok);
    }
}
