use crate::error::Result;
use crate::experiments::uutils::constants::COREUTILS_BINS_LIST;
use crate::experiments::uutils::model::UutilsExperiment;
use crate::experiments::uutils::utils::resolve_target;
use crate::utils::worker::Worker;

impl UutilsExperiment {
    /// Disables the uutils experiment by restoring backups and removing the package.
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
        // Package removal policy: uninstall uutils-* on disable.
        // NOTE: This removes the package even if it was present before `enable`.
        // To change this behavior, gate removal behind a CLI flag.
        if self.package_name.starts_with("uutils-") {
            log::info!("Removing package: {}", self.package_name);
            worker.remove_package(&self.package_name, assume_yes)?;
        } else {
            log::info!("Skipping removal of core package: {}", self.package_name);
        }
        Ok(())
    }
}
