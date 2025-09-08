use crate::error::Result;
use crate::utils::Distribution;
use crate::utils::worker::Worker;
use std::path::PathBuf;

/// Represents an experiment for replacing system utilities with uutils equivalents.
#[derive(Debug, Clone)]
pub struct UutilsExperiment {
    pub name: String,                    // e.g., "coreutils"
    pub package_name: String,            // e.g., "uutils-coreutils"
    pub unified_binary: Option<PathBuf>, // e.g., /usr/bin/coreutils
    pub bin_directory: PathBuf,          // e.g., /usr/lib/uutils/coreutils
}

impl UutilsExperiment {
    /// Checks if the current system is compatible with this experiment (Arch Linux).
    pub fn check_compatible<W: Worker>(&self, worker: &W) -> Result<bool> {
        let d: Distribution = worker.distribution()?;
        let id = d.id.to_ascii_lowercase();
        // Supported set with no gating among them: arch, manjaro, cachyos, endeavouros
        Ok(matches!(
            id.as_str(),
            "arch" | "manjaro" | "cachyos" | "endeavouros"
        ))
    }
}
