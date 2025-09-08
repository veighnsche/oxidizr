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
        if update_lists {
            log::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }
        
        // Check prerequisites and handle prompts
        check_download_prerequisites(worker, &self.package_name, assume_yes)?;
        // Visibility: AUR build for findutils will require checksums. These are expected to be provided
        // by the currently active coreutils (with checksum applets possibly flipped via --flip-checksums).
        match worker.which("sha256sum") {
            Ok(Some(p)) => {
                log::info!(
                    "AUR checksum preflight: using sha256sum at {} (provided by active coreutils)",
                    p.display()
                );
            }
            _ => {
                log::warn!(
                    "AUR checksum preflight: could not resolve 'sha256sum' in PATH; makepkg may fail"
                );
            }
        }
        
        // Install package
        log::info!("Installing package: {}", self.package_name);
        worker.install_package(&self.package_name, assume_yes)?;
        
        // Discover and link applets
        let applets = self.discover_applets(worker)?;
        if applets.is_empty() {
            log::error!(
                "❌ Expected: at least 1 findutils applet discovered after install; Received: 0"
            );
            return Err(Error::ExecutionFailed(format!(
                "❌ Expected: findutils applets discovered; Received: 0. Ensure {} is installed correctly.",
                self.package_name
            )));
        }
        log::info!(
            "✅ Expected: findutils applets discovered; Received: {}",
            applets.len()
        );
        
        log_applets_summary("findutils", &applets, 8);
        create_symlinks(worker, &applets, |name| self.resolve_target(name))?;
        
        Ok(())
    }
    
    pub fn disable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        if update_lists {
            log::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }
        
        // Restore findutils applets
        let targets = vec![self.resolve_target("find"), self.resolve_target("xargs")];
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
                log::warn!("No binary found for '{}' in known locations", name);
            }
        }
        
        // If nothing found, try to synthesize from known locations
        if applets.is_empty() {
            log::info!("Attempting to synthesize findutils applet locations...");
            std::fs::create_dir_all(&self.bin_directory).ok();
            
            for name in &known {
                let candidates = [
                    PathBuf::from(format!("/usr/bin/uu-{}", name)),
                    PathBuf::from(format!("/usr/lib/cargo/bin/{}", name)),
                ];
                
                if let Some(real) = candidates.iter().find(|p| p.exists()) {
                    let canonical_src = self.bin_directory.join(name);
                    if canonical_src.exists() {
                        std::fs::remove_file(&canonical_src).ok();
                    }
                    
                    match std::fs::copy(real, &canonical_src) {
                        Ok(_) => {
                            if let Ok(meta) = std::fs::metadata(real) {
                                let perm = meta.permissions();
                                std::fs::set_permissions(&canonical_src, perm).ok();
                            }
                            log::info!(
                                "Synthesized canonical source (copied) {} <- {}",
                                canonical_src.display(),
                                real.display()
                            );
                            applets.push((name.to_string(), canonical_src));
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to copy {} to canonical source: {}",
                                real.display(),
                                e
                            );
                        }
                    }
                }
            }
        }
        
        Ok(applets)
    }
    
    fn resolve_target(&self, filename: &str) -> PathBuf {
        resolve_usrbin(filename)
    }
}
