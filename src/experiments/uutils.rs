use crate::error::{CoreutilsError, Result};
use crate::utils::worker::Worker;
use crate::utils::Distribution;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct UutilsExperiment {
    pub name: String,                    // e.g., "coreutils"
    pub package: String,                 // e.g., "uutils-coreutils"
    pub unified_binary: Option<PathBuf>, // e.g., /usr/bin/coreutils
    pub bin_directory: PathBuf,          // e.g., /usr/lib/uutils/coreutils
}

impl UutilsExperiment {
    pub fn check_compatible<W: Worker>(&self, worker: &W) -> Result<bool> {
        let d: Distribution = worker.distribution()?;
        Ok(d.id.to_ascii_lowercase() == "arch")
    }

    pub fn enable<W: Worker>(&self, worker: &W, _assume_yes: bool, update_lists: bool) -> Result<()> {
        if !self.check_compatible(worker)? {
            return Err(CoreutilsError::Incompatible("Unsupported Arch release".into()));
        }
        if update_lists { log::info!("Updating package lists..."); worker.update_packages()?; }

        log::info!("Installing package: {}", self.package);
        worker.install_package(&self.package)?;

        // Determine applet names and their source paths
        let mut applets: Vec<(String, PathBuf)> = Vec::new();
        if self.name == "coreutils" {
            // Resolve unified coreutils dispatch binary robustly
            // Preference: configured path if it exists -> which("coreutils") -> default path
            let unified_path: PathBuf = if let Some(cfg) = &self.unified_binary {
                if cfg.exists() {
                    cfg.clone()
                } else if let Ok(Some(found)) = worker.which("coreutils") {
                    found
                } else {
                    PathBuf::from("/usr/bin/coreutils")
                }
            } else if let Ok(Some(found)) = worker.which("coreutils") {
                found
            } else {
                PathBuf::from("/usr/bin/coreutils")
            };
            if !unified_path.exists() {
                // Try multiple known candidate locations for the unified dispatch binary
                let candidates: [PathBuf; 3] = [
                    self.bin_directory.join("coreutils"),
                    PathBuf::from("/usr/lib/cargo/bin/coreutils"),
                    PathBuf::from("/usr/bin/coreutils.uutils"),
                ];
                if let Some(found) = candidates.iter().find(|p| p.exists()) {
                    log::warn!(
                        "Unified coreutils not found at {}; creating symlink /usr/bin/coreutils -> {}",
                        unified_path.display(),
                        found.display()
                    );
                    // Best-effort create; ignore errors so we can still proceed or fail clearly below
                    let _ = std::fs::create_dir_all(std::path::Path::new("/usr/bin"));
                    let _ = std::fs::remove_file("/usr/bin/coreutils");
                    if let Err(e) = std::os::unix::fs::symlink(found, "/usr/bin/coreutils") {
                        log::error!("Failed to create /usr/bin/coreutils symlink: {}", e);
                    } else {
                        log::info!("Created /usr/bin/coreutils symlink to {}", found.display());
                    }
                } else {
                    log::warn!(
                        "Unified coreutils binary not found in any known location ({}; {}; {}): will error if not present after this step",
                        self.bin_directory.join("coreutils").display(),
                        Path::new("/usr/lib/cargo/bin/coreutils").display(),
                        Path::new("/usr/bin/coreutils.uutils").display(),
                    );
                }
            }
            if Path::new("/usr/bin/coreutils").exists() {
                log::info!("Using unified coreutils binary at: {}", Path::new("/usr/bin/coreutils").display());
                // Use baked-in list of applets to symlink the unified binary to.
                const COREUTILS_BINS: &str = include_str!("../../tests/lib/rust-coreutils-bins.txt");
                for line in COREUTILS_BINS.lines() {
                    let name = line.trim();
                    if name.is_empty() { continue; }
                    applets.push((name.to_string(), Path::new("/usr/bin/coreutils").to_path_buf()));
                }
            } else {
                // Per-applet fallback: link each applet to its individual binary under bin_directory
                log::warn!(
                    "Unified dispatcher not available; falling back to per-applet binaries under {}",
                    self.bin_directory.display()
                );
                const COREUTILS_BINS: &str = include_str!("../../tests/lib/rust-coreutils-bins.txt");
                for line in COREUTILS_BINS.lines() {
                    let name = line.trim();
                    if name.is_empty() { continue; }
                    // Probe multiple candidate locations per applet
                    let candidates: [PathBuf; 4] = [
                        self.bin_directory.join(name),
                        PathBuf::from(format!("/usr/bin/uu-{}", name)),
                        PathBuf::from(format!("/usr/lib/cargo/bin/coreutils/{}", name)),
                        PathBuf::from(format!("/usr/lib/cargo/bin/{}", name)),
                    ];
                    if let Some(found) = candidates.iter().find(|p| p.exists()) {
                        log::info!(
                            "Per-applet source selected for '{}': {}",
                            name,
                            found.display()
                        );
                        applets.push((name.to_string(), found.clone()));
                    } else {
                        log::warn!(
                            "Per-applet binary for '{}' not found in any known location; skipping",
                            name
                        );
                    }
                }
                if applets.is_empty() {
                    return Err(CoreutilsError::ExecutionFailed(
                        format!("No coreutils applet binaries found under {}", self.bin_directory.display())
                    ));
                }
            }
        } else {
            // Use the files present in the bin_directory (e.g., findutils/xargs)
            log::info!("Listing replacement binaries in {}", self.bin_directory.display());
            let files = worker.list_files(&self.bin_directory)?;
            for f in files {
                let filename = f.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                if filename.is_empty() { continue; }
                applets.push((filename, f.clone()));
            }
        }

        for (filename, src) in applets {
            let target = resolve_target(worker, &filename);
            log::info!("Symlinking {} -> {}", src.display(), target.display());
            worker.replace_file_with_symlink(&src, &target)?;
        }
        Ok(())
    }

    pub fn disable<W: Worker>(&self, worker: &W, update_lists: bool) -> Result<()> {
        if update_lists { log::info!("Updating package lists..."); worker.update_packages()?; }

        // IMPORTANT: On enable() we may have linked a large, baked-in set of applets
        // to the unified dispatcher (/usr/bin/coreutils). Those applets will NOT be
        // present under bin_directory, so listing bin_directory is insufficient for
        // complete restoration. Mirror the enable() selection for coreutils.
        if self.name == "coreutils" {
            const COREUTILS_BINS: &str = include_str!("../../tests/lib/rust-coreutils-bins.txt");
            for line in COREUTILS_BINS.lines() {
                let filename = line.trim();
                if filename.is_empty() { continue; }
                let target = resolve_target(worker, filename);
                log::info!("[disable] Restoring {} (if backup exists)", target.display());
                worker.restore_file(&target)?;
            }
        } else {
            // Non-coreutils families: restore based on the binaries actually provided
            // by the package under bin_directory.
            let files = worker.list_files(&self.bin_directory)?;
            for f in files {
                let filename = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if filename.is_empty() { continue; }
                let target = resolve_target(worker, filename);
                log::info!("[disable] Restoring {} (if backup exists)", target.display());
                worker.restore_file(&target)?;
            }
        }
        log::info!("Removing package: {}", self.package);
        worker.remove_package(&self.package)?;
        Ok(())
    }

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
