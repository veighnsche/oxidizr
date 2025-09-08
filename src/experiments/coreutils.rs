use crate::checks::{Distribution, is_supported_distro};
use crate::error::{Error, Result};
use crate::experiments::{check_download_prerequisites, UUTILS_COREUTILS};
use crate::experiments::util::{create_symlinks, log_applets_summary, resolve_usrbin, restore_targets, verify_removed};
use crate::system::Worker;
use std::path::PathBuf;

// Coreutils bins list (same as original)
const COREUTILS_BINS_LIST: &str = include_str!("../../tests/lib/rust-coreutils-bins.txt");

// Binaries we must not replace to keep packaging tools functional (e.g., makepkg)
const PRESERVE_BINS: &[&str] = &[
    "b2sum",
    "md5sum",
    "sha1sum",
    "sha224sum",
    "sha256sum",
    "sha384sum",
    "sha512sum",
];

pub struct CoreutilsExperiment {
    name: String,
    package_name: String,
    unified_binary: Option<PathBuf>,
    bin_directory: PathBuf,
}

impl CoreutilsExperiment {
    pub fn new() -> Self {
        Self {
            name: "coreutils".to_string(),
            package_name: UUTILS_COREUTILS.to_string(),
            unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
            bin_directory: PathBuf::from("/usr/lib/uutils/coreutils"),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn check_compatible(&self, distro: &Distribution) -> Result<bool> {
        Ok(is_supported_distro(&distro.id))
    }
    
    pub fn enable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        if update_lists {
            log::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }
        
        // Check prerequisites and handle prompts
        check_download_prerequisites(worker, &self.package_name, assume_yes)?;
        
        // Install package
        log::info!("Installing package: {}", self.package_name);
        worker.install_package(&self.package_name, assume_yes)?;
        
        // Discover and link applets
        let applets = self.discover_applets(worker)?;
        if applets.is_empty() {
            log::error!(
                "❌ Expected: at least 1 coreutils applet discovered after install; Received: 0"
            );
            return Err(Error::ExecutionFailed(format!(
                "❌ Expected: coreutils applets discovered; Received: 0. Ensure {} is installed correctly.",
                self.package_name
            )));
        }
        log::info!(
            "✅ Expected: coreutils applets discovered; Received: {}",
            applets.len()
        );
        
        // Filter out preserved binaries (do not replace these targets)
        let original_len = applets.len();
        let filtered: Vec<(String, PathBuf)> = applets
            .into_iter()
            .filter(|(name, _)| !PRESERVE_BINS.contains(&name.as_str()))
            .collect();
        let preserved_count = original_len.saturating_sub(filtered.len());
        if preserved_count > 0 {
            log::info!(
                "Preserving {} checksum tool(s) unmodified: {:?}",
                preserved_count,
                PRESERVE_BINS
            );
        }
        
        log_applets_summary("coreutils", &filtered, 8);
        create_symlinks(worker, &filtered, |name| self.resolve_target(name))?;
        
        Ok(())
    }
    
    pub fn disable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        if update_lists {
            log::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }
        
        // Restore all coreutils applets
        let mut targets: Vec<PathBuf> = Vec::new();
        for line in COREUTILS_BINS_LIST.lines() {
            let filename = line.trim();
            if filename.is_empty() {
                continue;
            }
            let target = self.resolve_target(filename);
            targets.push(target);
        }
        restore_targets(worker, &targets)?;
        
        Ok(())
    }
    
    pub fn remove(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        // First restore GNU tools
        self.disable(worker, assume_yes, update_lists)?;
        
        // Then remove the package
        log::info!("Removing package: {}", self.package_name);
        worker.remove_package(&self.package_name, assume_yes)?;
        
        // Verify absence explicitly
        verify_removed(worker, &self.package_name)?;
        
        Ok(())
    }
    
    pub fn list_targets(&self) -> Vec<PathBuf> {
        let mut targets = Vec::new();
        for line in COREUTILS_BINS_LIST.lines() {
            let filename = line.trim();
            if !filename.is_empty() {
                targets.push(self.resolve_target(filename));
            }
        }
        targets
    }
    
    fn discover_applets(&self, worker: &Worker) -> Result<Vec<(String, PathBuf)>> {
        let mut applets = Vec::new();
        
        // Check for unified binary first
        let unified_path = if let Some(ref path) = self.unified_binary {
            if path.exists() {
                Some(path.clone())
            } else if let Ok(Some(found)) = worker.which("coreutils") {
                Some(found)
            } else {
                None
            }
        } else {
            None
        };
        
        if let Some(unified) = unified_path {
            log::info!("Using unified coreutils binary at: {}", unified.display());
            // Use unified binary for all applets
            for line in COREUTILS_BINS_LIST.lines() {
                let name = line.trim();
                if !name.is_empty() {
                    applets.push((name.to_string(), unified.clone()));
                }
            }
        } else {
            log::warn!("Unified dispatcher not available; falling back to per-applet binaries");
            // Try to find individual binaries
            for line in COREUTILS_BINS_LIST.lines() {
                let name = line.trim();
                if name.is_empty() {
                    continue;
                }
                
                // Try various locations
                let candidates = [
                    self.bin_directory.join(name),
                    PathBuf::from(format!("/usr/bin/uu-{}", name)),
                    PathBuf::from(format!("/usr/lib/cargo/bin/coreutils/{}", name)),
                    PathBuf::from(format!("/usr/lib/cargo/bin/{}", name)),
                ];
                
                if let Some(found) = candidates.iter().find(|p| p.exists()) {
                    applets.push((name.to_string(), found.clone()));
                } else if let Ok(Some(path)) = worker.which(name) {
                    applets.push((name.to_string(), path));
                }
            }
        }
        
        Ok(applets)
    }
    
    fn resolve_target(&self, filename: &str) -> PathBuf {
        resolve_usrbin(filename)
    }
}
