use crate::checks::{Distribution, is_supported_distro};
use crate::error::{Error, Result};
use crate::experiments::{check_download_prerequisites, UUTILS_FINDUTILS};
use crate::experiments::util::{create_symlinks, log_applets_summary, resolve_usrbin, restore_targets, verify_removed};
use crate::system::Worker;
use std::path::PathBuf;

pub struct FindutilsExperiment {
    name: String,
    package_name: String,
    bin_directory: PathBuf,
}

impl FindutilsExperiment {
    pub fn new() -> Self {
        Self {
            name: "findutils".to_string(),
            package_name: UUTILS_FINDUTILS.to_string(),
            bin_directory: PathBuf::from("/usr/lib/cargo/bin/findutils"),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn check_compatible(&self, distro: &Distribution) -> Result<bool> {
        Ok(is_supported_distro(&distro.id))
    }
    
    pub fn enable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        let _span = tracing::info_span!("findutils_enable", package = %self.package_name, update_lists).entered();
        if update_lists {
            tracing::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }
        
        // Check prerequisites and handle prompts
        check_download_prerequisites(worker, &self.package_name, assume_yes)?;
        // Visibility: AUR build for findutils will require checksums. These are expected to be provided
        // by the currently active coreutils (and optionally flipped via the dedicated 'checksums' experiment).
        match worker.which("sha256sum") {
            Ok(Some(p)) => {
                tracing::info!(
                    "AUR checksum preflight: using sha256sum at {} (provided by active coreutils)",
                    p.display()
                );
            }
            _ => {
                tracing::warn!(
                    "AUR checksum preflight: could not resolve 'sha256sum' in PATH; makepkg may fail"
                );
            }
        }
        
        // Install package
        tracing::info!("Installing package: {}", self.package_name);
        worker.install_package(&self.package_name, assume_yes)?;
        
        // Discover and link applets
        let applets = self.discover_applets(worker)?;
        if applets.is_empty() {
            let _ = crate::logging::audit_event(
                "experiments",
                "nothing_to_link",
                "findutils",
                "",
                "",
                None,
            );
            return Err(Error::NothingToLink(
                "no findutils applets discovered after install".into(),
            ));
        }
        tracing::info!(
            "✅ Expected: findutils applets discovered; Received: {}",
            applets.len()
        );
        
        log_applets_summary("findutils", &applets, 8);
        create_symlinks(worker, &applets, |name| self.resolve_target(name))?;
        
        Ok(())
    }
    
    pub fn disable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        let _span = tracing::info_span!("findutils_disable", package = %self.package_name, update_lists).entered();
        if update_lists {
            tracing::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }
        
        // Restore findutils applets
        let targets = vec![self.resolve_target("find"), self.resolve_target("xargs")];
        restore_targets(worker, &targets)?;
        
        Ok(())
    }
    
    pub fn remove(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        let _span = tracing::info_span!("findutils_remove", package = %self.package_name, update_lists).entered();
        // First restore GNU tools
        self.disable(worker, assume_yes, update_lists)?;
        
        // Then remove the package
        tracing::info!("Removing package: {}", self.package_name);
        worker.remove_package(&self.package_name, assume_yes)?;
        
        // Verify absence explicitly
        verify_removed(worker, &self.package_name)?;
        
        Ok(())
    }
    
    pub fn list_targets(&self) -> Vec<PathBuf> {
        vec![
            self.resolve_target("find"),
            self.resolve_target("xargs"),
        ]
    }
    
    fn discover_applets(&self, worker: &Worker) -> Result<Vec<(String, PathBuf)>> {
        let mut applets = Vec::new();
        let known = ["find", "xargs"];
        
        for name in &known {
            // Try various locations
            let candidates = [
                self.bin_directory.join(name),
                PathBuf::from(format!("/usr/bin/uu-{}", name)),
                PathBuf::from(format!("/usr/lib/cargo/bin/{}", name)),
                PathBuf::from(format!("/usr/bin/{}", name)),
            ];
            
            if let Some(found) = candidates.iter().find(|p| p.exists()) {
                applets.push((name.to_string(), found.clone()));
            } else if let Ok(Some(path)) = worker.which(name) {
                applets.push((name.to_string(), path));
            } else {
                tracing::warn!("No binary found for '{}' in known locations", name);
            }
        }
        
        Ok(applets)
    }
    
    fn resolve_target(&self, filename: &str) -> PathBuf {
        resolve_usrbin(filename)
    }
}
