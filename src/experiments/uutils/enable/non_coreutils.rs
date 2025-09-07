use crate::error::{CoreutilsError, Result};
use crate::experiments::uutils::model::UutilsExperiment;
use crate::utils::worker::Worker;
use std::env;
use std::path::Path;
use std::path::PathBuf;

impl UutilsExperiment {
    /// Handles applet collection for non-coreutils families.
    pub fn handle_non_coreutils_applets<W: Worker>(
        &self,
        worker: &W,
    ) -> Result<Vec<(String, PathBuf)>> {
        log::info!(
            "Preparing applets for family '{}' under {}",
            self.name,
            self.bin_directory.display()
        );
        let mut applets = Vec::new();
        let known: &[&str] = match self.name.as_str() {
            "findutils" => &["find", "xargs"],
            _ => &[],
        };

        if known.is_empty() {
            log::warn!(
                "No applets declared for family '{}' and no files under {}",
                self.name,
                self.bin_directory.display()
            );
            return Ok(applets);
        }

        for name in known {
            if let Ok(Some(path)) = worker.which(name) {
                applets.push((name.to_string(), path));
            } else {
                 log::warn!(
                    "No binary found for '{}' in known locations for family '{}'",
                    name,
                    self.name
                );
            }
        }

        if applets.is_empty() {
            let known: &[&str] = match self.name.as_str() {
                "findutils" => &["find", "xargs"],
                _ => &[],
            };
            if known.is_empty() {
                log::warn!(
                    "No applets declared for family '{}' and no files under {}",
                    self.name,
                    self.bin_directory.display()
                );
            }
            if let Some(parent) = self.bin_directory.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::create_dir_all(&self.bin_directory);
            for name in known {
                let candidates = self.get_non_coreutils_candidates(name);
                if let Some(real) = candidates.iter().find(|p| p.exists()) {
                    let canonical_src = self.bin_directory.join(name);
                    if canonical_src.exists() {
                        let _ = std::fs::remove_file(&canonical_src);
                    }
                    match std::fs::copy(real, &canonical_src) {
                        Ok(_) => {
                            if let Ok(meta) = std::fs::metadata(real) {
                                let perm = meta.permissions();
                                let _ = std::fs::set_permissions(&canonical_src, perm);
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
                                "Failed to copy {} to canonical source {}: {}",
                                real.display(),
                                canonical_src.display(),
                                e
                            );
                        }
                    }
                } else {
                    log::warn!(
                        "No binary found for '{}' in known locations for family '{}'",
                        name,
                        self.name
                    );
                }
            }
            if applets.is_empty() {
                return Err(CoreutilsError::ExecutionFailed(format!(
                    "No '{}' applet binaries found or synthesized under {}. \
                         Ensure '{}' installed correctly; if installed via AUR, verify that the helper completed successfully.",
                    self.name,
                    self.bin_directory.display(),
                    self.package_name
                )));
            }
        }
        Ok(applets)
    }

    /// Gets candidate paths for non-coreutils binaries.
    pub fn get_non_coreutils_candidates(&self, name: &str) -> [PathBuf; 4] {
        if cfg!(test) {
            let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
            let base_dir = Path::new(&out_dir).join("bin");
            [
                base_dir.join(format!("uu-{}", name)),
                base_dir.join(format!("{}/{}", self.name, name)),
                base_dir.join(name),
                base_dir.join(name),
            ]
        } else {
            [
                PathBuf::from(format!("/usr/bin/uu-{}", name)),
                PathBuf::from(format!("/usr/lib/cargo/bin/{}/{}", self.name, name)),
                PathBuf::from(format!("/usr/lib/cargo/bin/{}", name)),
                PathBuf::from(format!("/usr/bin/{}", name)),
            ]
        }
    }
}
