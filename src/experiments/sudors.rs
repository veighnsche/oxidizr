use crate::checks::{is_supported_distro, Distribution};
use crate::error::{Error, Result};
use crate::experiments::util::{resolve_usrbin, restore_targets, verify_removed};
use crate::experiments::{check_download_prerequisites, SUDO_RS};
use crate::system::Worker;
use crate::ui::progress;
use std::path::PathBuf;

pub struct SudoRsExperiment {
    name: String,
    package_name: String,
}

impl SudoRsExperiment {
    pub fn new() -> Self {
        Self {
            name: "sudo-rs".to_string(),
            package_name: SUDO_RS.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn check_compatible(&self, distro: &Distribution) -> Result<bool> {
        Ok(is_supported_distro(&distro.id))
    }

    pub fn enable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        let _span =
            tracing::info_span!("sudors_enable", package = %self.package_name, update_lists)
                .entered();
        if update_lists {
            tracing::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }

        // Check prerequisites and handle prompts
        check_download_prerequisites(worker, &self.package_name, assume_yes)?;

        // Install package
        tracing::info!(event = "package_install", package = %self.package_name, "Installing package: {}", self.package_name);
        worker.install_package(&self.package_name, assume_yes)?;
        if worker.check_installed(&self.package_name)? {
            tracing::info!(
                "✅ Expected: '{}' installed, Received: present",
                self.package_name
            );
        } else {
            tracing::error!(
                "❌ Expected: '{}' installed, Received: absent",
                self.package_name
            );
            return Err(Error::ExecutionFailed(format!(
                "❌ Expected: '{}' installed, Received: absent",
                self.package_name
            )));
        }

        // Replace sudo, su, visudo with binaries provided by sudo-rs
        let items = [
            ("sudo", self.resolve_target("sudo")),
            ("su", self.resolve_target("su")),
            ("visudo", PathBuf::from("/usr/sbin/visudo")),
        ];
        let pb = progress::new_bar(items.len() as u64);
        let _quiet_guard = if pb.is_some() {
            Some(progress::enable_symlink_quiet())
        } else {
            None
        };
        for (name, target) in items {
            tracing::info!("Preparing sudo-rs applet '{}'", name);

            let source = self.find_sudors_source(worker, name);
            let source = source.ok_or_else(|| {
                Error::ExecutionFailed(format!(
                    "Could not find installed sudo-rs binary for '{0}'. \
                     Checked: /usr/lib/cargo/bin/{0} and /usr/bin/{0}-rs. \
                     Hints: ensure 'sudo-rs' is installed and provides '{0}' on this distro.",
                    name
                ))
            })?;

            // Create a stable alias in /usr/bin so that readlink(1) shows '/usr/bin/<name>.sudo-rs'
            let alias = PathBuf::from(format!("/usr/bin/{}.sudo-rs", name));
            if pb.is_none() {
                tracing::info!(
                    "Creating alias for sudo-rs '{}': {} -> {}",
                    name,
                    alias.display(),
                    source.display()
                );
            }
            worker.replace_file_with_symlink(&source, &alias)?;
            // Verify alias symlink presence for visibility; treat mismatches as hard errors
            match std::fs::symlink_metadata(&alias) {
                Ok(m) if m.file_type().is_symlink() => {
                    if pb.is_none() {
                        tracing::info!(
                            "✅ Expected: '{}' alias symlink present, Received: symlink",
                            name
                        );
                    }
                }
                Ok(_) => {
                    return Err(Error::ExecutionFailed(format!(
                        "alias for '{}' not a symlink: {}",
                        name,
                        alias.display()
                    )));
                }
                Err(e) => {
                    return Err(Error::ExecutionFailed(format!(
                        "alias for '{}' missing: {} (err: {})",
                        name,
                        alias.display(),
                        e
                    )));
                }
            }

            if pb.is_none() {
                tracing::info!(
                    "Linking sudo-rs '{}' via alias: {} -> {}",
                    name,
                    target.display(),
                    alias.display()
                );
            }
            worker.replace_file_with_symlink(&alias, &target)?;
            // Verify target symlink presence; treat mismatches as hard errors
            match std::fs::symlink_metadata(&target) {
                Ok(m) if m.file_type().is_symlink() => {
                    if pb.is_none() {
                        tracing::info!(
                            "✅ Expected: '{}' linked via alias, Received: symlink",
                            name
                        );
                    }
                }
                Ok(_) => {
                    return Err(Error::ExecutionFailed(format!(
                        "target for '{}' not a symlink: {}",
                        name,
                        target.display()
                    )));
                }
                Err(e) => {
                    return Err(Error::ExecutionFailed(format!(
                        "target for '{}' missing: {} (err: {})",
                        name,
                        target.display(),
                        e
                    )));
                }
            }

            // Update bar after finishing both alias and target for this name
            progress::set_msg_and_inc(&pb, format!("Linking {}", name));
        }
        progress::finish(pb);

        Ok(())
    }

    pub fn disable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        let _span =
            tracing::info_span!("sudors_disable", package = %self.package_name, update_lists)
                .entered();
        if update_lists {
            tracing::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }

        // Restore original binaries (fail fast on mismatches)
        let targets = vec![
            self.resolve_target("sudo"),
            self.resolve_target("su"),
            PathBuf::from("/usr/sbin/visudo"),
        ];
        restore_targets(worker, &targets)?;
        // Verify restored (not a symlink)
        for (name, target) in [
            ("sudo", &targets[0]),
            ("su", &targets[1]),
            ("visudo", &targets[2]),
        ] {
            match std::fs::symlink_metadata(target) {
                Ok(m) if m.file_type().is_symlink() => {
                    return Err(Error::ExecutionFailed(format!(
                        "{} was expected restored to non-symlink but is still a symlink: {}",
                        name,
                        target.display()
                    )));
                }
                Ok(_) => {
                    tracing::info!(
                        "✅ Expected: '{}' restored to non-symlink, Received: non-symlink",
                        name
                    );
                }
                Err(e) => {
                    return Err(Error::ExecutionFailed(format!(
                        "{} missing after restore: {} (err: {})",
                        name,
                        target.display(),
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn remove(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        let _span =
            tracing::info_span!("sudors_remove", package = %self.package_name, update_lists)
                .entered();
        // First restore GNU tools
        self.disable(worker, assume_yes, update_lists)?;

        // Then remove the package
        tracing::info!(event = "package_remove", package = %self.package_name, "Removing package: {}", self.package_name);
        worker.remove_package(&self.package_name, assume_yes)?;

        // Verify absence
        verify_removed(worker, &self.package_name)?;

        Ok(())
    }

    pub fn list_targets(&self) -> Vec<PathBuf> {
        vec![
            self.resolve_target("sudo"),
            self.resolve_target("su"),
            PathBuf::from("/usr/sbin/visudo"),
        ]
    }

    fn find_sudors_source(&self, worker: &Worker, name: &str) -> Option<PathBuf> {
        // Always resolve to sudo-rs-provided binaries. Do not fall back to the system 'sudo'.
        // Prefer explicit locations, then consult PATH for '*-rs'.
        let rs_name = format!("{}-rs", name);
        let candidates = [
            PathBuf::from(format!("/usr/lib/cargo/bin/{}", name)),
            PathBuf::from(format!("/usr/bin/{}", rs_name)),
        ];

        for c in candidates {
            tracing::debug!("checking sudo-rs candidate for '{}': {}", name, c.display());
            if c.exists() {
                return Some(c);
            }
        }

        if let Ok(Some(path)) = worker.which(&rs_name) {
            tracing::debug!("found sudo-rs on PATH for '{}': {}", name, path.display());
            return Some(path);
        }

        None
    }

    fn resolve_target(&self, filename: &str) -> PathBuf {
        resolve_usrbin(filename)
    }
}
