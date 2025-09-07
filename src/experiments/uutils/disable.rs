use crate::error::Result;
use crate::experiments::uutils::constants::COREUTILS_BINS_LIST;
use crate::experiments::uutils::model::UutilsExperiment;
use crate::experiments::uutils::utils::resolve_target;
use crate::utils::worker::Worker;

impl UutilsExperiment {
    /// Disables the uutils experiment by restoring backups and removing the package.
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
        if self.package_name.starts_with("uutils-") {
            log::info!("Removing package: {}", self.package_name);
            worker.remove_package(&self.package_name, assume_yes)?;
        } else {
            log::info!("Skipping removal of core package: {}", self.package_name);
        }
        Ok(())
    }
}
