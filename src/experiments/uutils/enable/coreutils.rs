use crate::error::Result;
use crate::experiments::uutils::constants::{
    COREUTILS_BINS_LIST, COREUTILS_UNIFIED_CANDIDATES, COREUTILS_UNIFIED_PATH, SYSTEM_BIN_DIR,
};
use crate::experiments::uutils::model::UutilsExperiment;
use crate::utils::worker::Worker;
use std::env;
use std::path::{Path, PathBuf};

impl UutilsExperiment {
    /// Handles applet collection for coreutils family.
    pub fn handle_coreutils_applets<W: Worker>(
        &self,
        worker: &W,
    ) -> Result<Vec<(String, PathBuf)>> {
        let _unified_path = self.resolve_unified_coreutils_path(worker);
        if Path::new(if cfg!(test) {
            "bin/coreutils"
        } else {
            COREUTILS_UNIFIED_PATH
        })
        .exists()
        {
            log::info!(
                "Using unified coreutils binary at: {}",
                if cfg!(test) {
                    "bin/coreutils"
                } else {
                    COREUTILS_UNIFIED_PATH
                }
            );
            Ok(self.collect_applets_with_unified_dispatcher(worker))
        } else {
            log::warn!(
                "Unified dispatcher not available; falling back to per-applet binaries under {}",
                self.bin_directory.display()
            );
            Ok(self.collect_applets_per_binary(worker))
        }
    }

    /// Resolves the path to the unified coreutils binary.
    pub fn resolve_unified_coreutils_path<W: Worker>(&self, worker: &W) -> PathBuf {
        if let Some(cfg) = &self.unified_binary {
            if cfg.exists() {
                return cfg.clone();
            } else if let Ok(Some(found)) = worker.which("coreutils") {
                return found;
            }
        } else if let Ok(Some(found)) = worker.which("coreutils") {
            return found;
        }

        let default_path = if cfg!(test) {
            let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(out_dir).join("bin/coreutils")
        } else {
            PathBuf::from(COREUTILS_UNIFIED_PATH)
        };
        if !default_path.exists() {
            if let Some(found) = COREUTILS_UNIFIED_CANDIDATES
                .iter()
                .map(|p| {
                    if cfg!(test) {
                        PathBuf::from("bin").join(p.trim_start_matches('/'))
                    } else {
                        PathBuf::from(p)
                    }
                })
                .find(|p| p.exists())
            {
                log::warn!(
                    "Unified coreutils not found at {}; creating symlink {} -> {}",
                    default_path.display(),
                    if cfg!(test) {
                        "bin/coreutils"
                    } else {
                        COREUTILS_UNIFIED_PATH
                    },
                    found.display()
                );
                let target_dir = if cfg!(test) {
                    Path::new("bin")
                } else {
                    Path::new(SYSTEM_BIN_DIR)
                };
                let _ = std::fs::create_dir_all(target_dir);
                let target_path = if cfg!(test) {
                    PathBuf::from("bin/coreutils")
                } else {
                    PathBuf::from(COREUTILS_UNIFIED_PATH)
                };
                let _ = std::fs::remove_file(&target_path);
                if let Err(e) = std::os::unix::fs::symlink(&found, &target_path) {
                    log::error!("Failed to create {} symlink: {}", target_path.display(), e);
                } else {
                    log::info!(
                        "Created {} symlink to {}",
                        target_path.display(),
                        found.display()
                    );
                }
            } else {
                log::warn!(
                    "Unified coreutils binary not found in any known location: will error if not present after this step"
                );
            }
        }
        default_path
    }

    /// Collects applets using a unified dispatcher binary.
    pub fn collect_applets_with_unified_dispatcher<W: Worker>(
        &self,
        worker: &W,
    ) -> Vec<(String, PathBuf)> {
        let mut applets = Vec::new();
        for line in COREUTILS_BINS_LIST.lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }
            if name == "tsor" {
                if let Ok(Some(tsort_target)) = worker.which("tsort") {
                    applets.push(("tsor".to_string(), tsort_target));
                    continue;
                }
                if let Ok(Some(tsor_target)) = worker.which("tsor") {
                    applets.push(("tsor".to_string(), tsor_target));
                    continue;
                }
                applets.push((
                    "tsor".to_string(),
                    if cfg!(test) {
                        PathBuf::from("bin/coreutils")
                    } else {
                        PathBuf::from(COREUTILS_UNIFIED_PATH)
                    },
                ));
                continue;
            }
            if name == "arch" {
                applets.push((
                    name.to_string(),
                    if cfg!(test) {
                        PathBuf::from("bin/coreutils")
                    } else {
                        PathBuf::from(COREUTILS_UNIFIED_PATH)
                    },
                ));
                continue;
            }
            applets.push((
                name.to_string(),
                if cfg!(test) {
                    PathBuf::from("bin/coreutils")
                } else {
                    PathBuf::from(COREUTILS_UNIFIED_PATH)
                },
            ));
        }
        applets
    }

    /// Collects applets by linking to individual binaries.
    pub fn collect_applets_per_binary<W: Worker>(&self, worker: &W) -> Vec<(String, PathBuf)> {
        let mut applets = Vec::new();
        for line in COREUTILS_BINS_LIST.lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }
            let (probe_name, link_as) = if name == "tsor" {
                ("tsort", "tsor")
            } else {
                (name, name)
            };
            if link_as == "arch" {
                let mut candidates = self.get_coreutils_candidates(probe_name).to_vec();
                // Filter out candidates that are already symlinked to the unified binary
                let unified_binary_name = if cfg!(test) {
                    "bin/coreutils"
                } else {
                    COREUTILS_UNIFIED_PATH
                };
                candidates.retain(|candidate| {
                    if let Ok(dest) = std::fs::read_link(candidate) {
                        !dest.ends_with(unified_binary_name)
                    } else {
                        true
                    }
                });
                if let Some(found) = candidates.iter().find(|p| p.exists()) {
                    log::info!(
                        "Per-applet source selected for '{}': {}",
                        link_as,
                        found.display()
                    );
                    applets.push((link_as.to_string(), found.clone()));
                } else {
                    log::warn!(
                        "Per-applet binary for 'arch' not found in any known location; using fallback path"
                    );
                    // Use a relative path in test mode
                    let fallback_path = if cfg!(test) {
                        let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
                        PathBuf::from(out_dir).join("bin/uu-arch")
                    } else {
                        PathBuf::from("/usr/bin/uu-arch")
                    };
                    applets.push((link_as.to_string(), fallback_path));
                }
                continue;
            }
            if worker.which(link_as).unwrap_or(None).is_none() {
                log::debug!("Skipping '{}' (no discoverable target via which)", link_as);
                continue;
            }
            let candidates = self.get_coreutils_candidates(probe_name);
            if let Some(found) = candidates.iter().find(|p| p.exists()) {
                log::info!(
                    "Per-applet source selected for '{}': {}",
                    link_as,
                    found.display()
                );
                applets.push((link_as.to_string(), found.clone()));
            } else {
                log::warn!(
                    "Per-applet binary for '{}' not found in any known location; skipping",
                    link_as
                );
            }
        }
        applets
    }

    /// Gets candidate paths for coreutils binaries.
    pub fn get_coreutils_candidates(&self, probe_name: &str) -> [PathBuf; 4] {
        if cfg!(test) {
            [
                self.bin_directory.join(probe_name),
                PathBuf::from(format!("bin/uu-{}", probe_name)),
                PathBuf::from(format!("bin/coreutils/{}", probe_name)),
                PathBuf::from(format!("bin/{}", probe_name)),
            ]
        } else {
            [
                self.bin_directory.join(probe_name),
                PathBuf::from(format!("/usr/bin/uu-{}", probe_name)),
                PathBuf::from(format!("/usr/lib/cargo/bin/coreutils/{}", probe_name)),
                PathBuf::from(format!("/usr/lib/cargo/bin/{}", probe_name)),
            ]
        }
    }
}
