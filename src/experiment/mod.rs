use crate::error::{CoreutilsError, Result};
use crate::worker::Worker;
use std::path::{Path, PathBuf};

/// Describes a family of utilities (e.g., coreutils) and how to switch them on Arch Linux.
#[derive(Debug, Clone)]
pub struct UutilsExperiment {
    pub name: String,                    // e.g., "coreutils"
    pub package: String,                 // e.g., "uutils-coreutils" (AUR/extra)
    pub supported_releases: Vec<String>, // e.g., ["rolling"]
    pub unified_binary: Option<PathBuf>, // e.g., /usr/bin/coreutils
    pub bin_directory: PathBuf,          // e.g., /usr/lib/uutils/coreutils
}

impl UutilsExperiment {
    pub fn check_compatible<W: Worker>(&self, worker: &W) -> Result<bool> {
        let (distro, release) = worker.distribution()?;
        if distro != "Arch" {
            return Ok(false);
        }
        Ok(self.supported_releases.iter().any(|r| r == &release))
    }

    /// Enable flow (scaffold): install package, list files, compute targets, swap-in symlinks.
    /// This is a placeholder that wires the steps but leaves bodies minimal.
    pub fn enable<W: Worker>(&self, worker: &W, _assume_yes: bool, update_lists: bool) -> Result<()> {
        // Safety gates
        if !self.check_compatible(worker)? {
            return Err(CoreutilsError::Incompatible("Unsupported Arch release".into()));
        }
        if update_lists { log::info!("Updating package lists..."); worker.update_packages()?; }
        // In real impl, verify root and confirm unless assume_yes

        // Install the replacement package
        log::info!("Installing package: {}", self.package);
        worker.install_package(&self.package)?;

        // Discover binaries
        log::info!("Listing replacement binaries in {}", self.bin_directory.display());
        let files = worker.list_files(&self.bin_directory)?;
        for f in files {
            let filename = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if filename.is_empty() { continue; }

            let target = resolve_target(worker, filename);
            log::info!("Computed target for {} -> {}", filename, target.display());

            match &self.unified_binary {
                Some(unified) => {
                    // symlink unified -> target
                    log::info!("Symlinking unified {} -> {}", unified.display(), target.display());
                    worker.replace_file_with_symlink(unified, &target)?;
                }
                None => {
                    // symlink per-binary file -> target
                    log::info!("Symlinking {} -> {}", f.display(), target.display());
                    worker.replace_file_with_symlink(&f, &target)?;
                }
            }
        }
        Ok(())
    }

    /// Disable flow (scaffold): restore backups and remove package.
    pub fn disable<W: Worker>(&self, worker: &W, update_lists: bool) -> Result<()> {
        if update_lists { log::info!("Updating package lists..."); worker.update_packages()?; }

        log::info!("Listing replacement binaries in {}", self.bin_directory.display());
        let files = worker.list_files(&self.bin_directory)?;
        for f in files {
            let filename = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if filename.is_empty() { continue; }
            let target = resolve_target(worker, filename);
            log::info!("Restoring target {} (if backup exists)", target.display());
            worker.restore_file(&target)?;
        }

        log::info!("Removing package: {}", self.package);
        worker.remove_package(&self.package)?;
        Ok(())
    }

    /// Compute and return the candidate target paths for inspection.
    pub fn list_targets<W: Worker>(&self, worker: &W) -> Result<Vec<PathBuf>> {
        let files = worker.list_files(&self.bin_directory)?;
        let mut out = Vec::new();
        for f in files {
            let filename = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if filename.is_empty() { continue; }
            out.push(resolve_target(worker, filename));
        }
        Ok(out)
    }
}

fn resolve_target<W: Worker>(worker: &W, filename: &str) -> PathBuf {
    if let Ok(Some(path)) = worker.which(filename) {
        path
    } else {
        Path::new("/usr/bin").join(filename)
    }
}
