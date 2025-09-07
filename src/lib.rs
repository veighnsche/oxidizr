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
    use super::*;
    use crate::experiments::UutilsExperiment;
    use crate::utils::worker::{System, Worker};
    use std::path::PathBuf;

    #[test]
    fn test_check_compatible_scaffold() {
        let exp = UutilsExperiment {
            name: "coreutils".into(),
            package: "uutils-coreutils".into(),
            unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
            bin_directory: PathBuf::from("/usr/lib/uutils/coreutils"),
        };
        let sys = System {
            aur_helper: "paru".into(),
            dry_run: true,
            wait_lock_secs: None,
        };
        let ok = exp.check_compatible(&sys).expect("compat check");
        assert!(ok);
    }
}
