use crate::error::{CoreutilsError, Result};
use crate::utils::worker::Worker;
use crate::experiments::uutils::constants::{COREUTILS_UNIFIED_PATH, COREUTILS_UNIFIED_CANDIDATES, COREUTILS_BINS_LIST};
use crate::experiments::uutils::model::UutilsExperiment;
use std::path::{Path, PathBuf};
use std::fs;

impl UutilsExperiment {
    /// Enables the uutils experiment by installing the package and setting up symlinks.
    pub fn enable<W: Worker>(&self, worker: &W, _assume_yes: bool, update_lists: bool) -> Result<()> {
        if !self.check_compatible(worker)? {
            return Err(CoreutilsError::Incompatible("Unsupported Arch release".into()));
        }
        if update_lists { log::info!("Updating package lists..."); worker.update_packages()?; }

        log::info!("Installing package: {}", self.package);
        worker.install_package(&self.package)?;

        let applets = if self.name == "coreutils" {
            self.handle_coreutils_applets(worker)?
        } else {
            self.handle_non_coreutils_applets(worker)?
        };

        if applets.is_empty() {
            return Err(CoreutilsError::ExecutionFailed(format!(
                "No applets selected for family '{}' (bin_directory: {}). This usually means the package did not install binaries in expected locations. \
                 Hints: ensure '{}' is installed; verify presence under {} or cargo-style /usr/lib/cargo/bin/<family>/.",
                self.name,
                self.bin_directory.display(),
                self.package,
                self.bin_directory.display()
            )));
        }

        self.log_applets_summary(&applets);
        self.create_symlinks(worker, &applets)
    }
}
