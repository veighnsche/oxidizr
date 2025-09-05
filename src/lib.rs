//! A scaffolding library for safely switching system core utilities
//! (oxidizr-arch inspired). This is intentionally minimal and non-destructive.

pub mod core; // placeholder for future per-command abstractions
pub mod error;
pub mod cli;
pub mod worker;
pub mod experiment;

// Re-export commonly used items
pub use crate::error::{CoreutilsError, Result};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experiment::UutilsExperiment;
    use crate::worker::{System, Worker};
    use std::path::PathBuf;

    #[test]
    fn test_check_compatible_scaffold() {
        let exp = UutilsExperiment {
            name: "coreutils".into(),
            package: "uutils-coreutils".into(),
            supported_releases: vec!["rolling".into()],
            unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
            bin_directory: PathBuf::from("/usr/lib/uutils/coreutils"),
        };
        let sys = System { aur_helper: "paru".into() };
        let ok = exp.check_compatible(&sys).expect("compat check");
        assert!(ok);
    }
}
