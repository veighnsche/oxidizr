use crate::config::packages;
use crate::error::Result;
use crate::experiments::uutils::constants::COREUTILS_BINS_LIST;
use crate::experiments::uutils::model::UutilsExperiment;
use crate::experiments::uutils::utils::resolve_target;
use crate::utils::worker::Worker;

impl UutilsExperiment {
    /// Disables the uutils experiment by restoring backups (does not uninstall the package).
    ///
    /// Semantics:
    /// - Restores any targets (e.g., `/usr/bin/<applet>`) from side-by-side backups
    ///   created during `enable`.
    /// - If the package name starts with `uutils-`, we uninstall it via the worker,
    ///   regardless of whether it was preinstalled by the user or installed by a
    ///   prior `enable` run. If a more conservative behavior is desired (e.g., only
    ///   uninstall when we installed it), introduce a CLI flag (e.g., `--purge`) and
    ///   gate this call accordingly.
    pub fn disable<W: Worker>(&self, worker: &W, assume_yes: bool, update_lists: bool) -> Result<()> {
        if update_lists {
            log::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }

        if self.name == "coreutils" {
            for line in COREUTILS_BINS_LIST.lines() {
                let filename = line.trim();
                if filename.is_empty() {
                    continue;
                }
                let target = resolve_target(worker, filename);
                log::info!(
                    "[disable] Restoring {} (if backup exists)",
                    target.display()
                );
                worker.restore_file(&target)?;
            }
        } else {
            let files = worker.list_files(&self.bin_directory)?;
            for f in files {
                let filename = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if filename.is_empty() {
                    continue;
                }
                let target = resolve_target(worker, filename);
                log::info!(
                    "[disable] Restoring {} (if backup exists)",
                    target.display()
                );
                worker.restore_file(&target)?;
            }
        }
        Ok(())
    }

    /// Removes the uutils package for this experiment (after restoring backups).
    /// Explicitly removes only the exact package per policy: no wildcards.
    pub fn remove<W: Worker>(&self, worker: &W, assume_yes: bool, update_lists: bool) -> Result<()> {
        // Always restore first to ensure GNU is back in place
        self.disable(worker, assume_yes, update_lists)?;
        match self.name.as_str() {
            "coreutils" => {
                log::info!("Removing package: {}", packages::UUTILS_COREUTILS);
                worker.remove_package(packages::UUTILS_COREUTILS, assume_yes)?;
            }
            "findutils" => {
                log::info!("Removing package: {}", packages::UUTILS_FINDUTILS);
                worker.remove_package(packages::UUTILS_FINDUTILS, assume_yes)?;
            }
            _ => {
                log::info!(
                    "No removable uutils package declared for family '{}' (skipping)",
                    self.name
                );
            }
        }
        Ok(())
    }
}
