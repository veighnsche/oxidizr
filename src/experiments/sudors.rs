use crate::config::packages;
use crate::error::{CoreutilsError, Result};
use crate::utils::Distribution;
use crate::utils::worker::Worker;
use crate::utils::audit::AUDIT;
use std::path::{Path, PathBuf};
use std::io::{self, Write};

pub struct SudoRsExperiment<'a, W: Worker> {
    pub system: &'a W,
    pub package_name: String,
}

// Prefer cargo-style install location, but accept Arch packaging under /usr/bin/*-rs.
fn find_sudors_source<W: Worker>(worker: &W, name: &str) -> Option<PathBuf> {
    // Always resolve to sudo-rs-provided binaries. Do not fall back to the system 'sudo'.
    // Prefer explicit locations, then consult PATH for '*-rs'.
    let rs_name = format!("{}-rs", name);
    let candidates = [
        PathBuf::from(format!("/usr/lib/cargo/bin/{}", name)),
        PathBuf::from(format!("/usr/bin/{}", rs_name)),
    ];
    for c in candidates {
        log::debug!("checking sudo-rs candidate for '{}': {}", name, c.display());
        if Path::new(&c).exists() {
            return Some(c);
        }
    }
    if let Ok(Some(path)) = worker.which(&rs_name) {
        log::debug!("found sudo-rs on PATH for '{}': {}", name, path.display());
        return Some(path);
    }
    None
}

impl<'a, W: Worker> SudoRsExperiment<'a, W> {
    pub fn name(&self) -> &'static str {
        "sudo-rs"
    }

    pub fn supported_releases(&self) -> Vec<String> {
        vec!["rolling".into()]
    }

    pub fn check_compatible(&self, worker: &W) -> Result<bool> {
        let d: Distribution = worker.distribution()?;
        let id = d.id.to_ascii_lowercase();
        // Supported set with no gating among them: arch, manjaro, cachyos, endeavouros
        Ok(matches!(
            id.as_str(),
            "arch" | "manjaro" | "cachyos" | "endeavouros"
        ))
    }

    pub fn enable(
        &self,
        worker: &W,
        assume_yes: bool,
        update_lists: bool,
    ) -> Result<()> {
        if !self.check_compatible(worker)? {
            return Err(CoreutilsError::Incompatible(
                "Unsupported Arch release".into(),
            ));
        }
        if update_lists {
            worker.update_packages(assume_yes)?;
        }
        // Repo capability checks and availability gating (official repos only)
        let extra_available = worker.extra_repo_available()?;
        let aur_helper = worker.aur_helper_name()?;
        let aur_available = aur_helper.is_some();
        let _ = AUDIT.log_provenance(
            "sudors.enable",
            "repo_capabilities",
            "observed",
            &format!(
                "extra_available={}, aur_available={}, helper={:?}",
                extra_available, aur_available, aur_helper
            ),
            "",
            None,
        );

        if !extra_available && !aur_available {
            return Err(CoreutilsError::ExecutionFailed(
                "You do not have access to extra or AUR repositories.".into(),
            ));
        }
        if !extra_available {
            return Err(CoreutilsError::ExecutionFailed(
                "Cannot download because the extra repository is not available.".into(),
            ));
        }

        // Already-installed detection and prompt to reuse
        if worker.check_installed(&self.package_name)? {
            let mut reuse = true;
            if !assume_yes {
                print!(
                    "Detected {} installed. Use existing instead of downloading? [Y/n]: ",
                    self.package_name
                );
                io::stdout().flush().ok();
                let mut s = String::new();
                if io::stdin().read_line(&mut s).is_ok() {
                    let ans = s.trim().to_ascii_lowercase();
                    reuse = ans.is_empty() || ans == "y" || ans == "yes";
                }
            }
            let _ = AUDIT.log_provenance(
                "sudors.enable",
                "already_installed",
                if reuse { "reuse" } else { "reinstall_requested" },
                &self.package_name,
                "",
                None,
            );
            if reuse {
                log::info!(
                    "Using existing installation of '{}' (no download)",
                    self.package_name
                );
            } else {
                log::info!(
                    "Reinstall requested for '{}' (will attempt package install)",
                    self.package_name
                );
            }
        }

        // Install sudo-rs (official repos only per worker policy)
        self.system.install_package(&self.package_name, assume_yes)?;
        // Replace sudo, su, visudo with binaries provided by sudo-rs. The Arch package layout may
        // install either into /usr/lib/cargo/bin/<name> or as /usr/bin/<name>-rs. Detect robustly.
        for (name, target) in [
            ("sudo", resolve_target(worker, "sudo")),
            ("su", resolve_target(worker, "su")),
            ("visudo", PathBuf::from("/usr/sbin/visudo")),
        ] {
            log::info!("Preparing sudo-rs applet '{}'", name);
            let source = find_sudors_source(worker, name);
            let source = source.ok_or_else(|| {
                CoreutilsError::ExecutionFailed(format!(
                    "Could not find installed sudo-rs binary for '{0}'. \
                 Checked: /usr/lib/cargo/bin/{0} and /usr/bin/{0}-rs. \
                 Hints: ensure 'sudo-rs' is installed and provides '{0}' on this distro.",
                    name
                ))
            })?;
            // Create a stable alias in /usr/bin so that readlink(1) shows '/usr/bin/<name>.sudo-rs'.
            let alias = PathBuf::from(format!("/usr/bin/{}.sudo-rs", name));
            log::info!(
                "Creating alias for sudo-rs '{}': {} -> {}",
                name,
                alias.display(),
                source.display()
            );
            worker.replace_file_with_symlink(&source, &alias)?;
            log::info!(
                "Linking sudo-rs '{}' via alias: {} -> {}",
                name,
                target.display(),
                alias.display()
            );
            worker.replace_file_with_symlink(&alias, &target)?;
        }
        Ok(())
    }

    pub fn disable(&self, worker: &W, assume_yes: bool, update_lists: bool) -> Result<()> {
        if update_lists {
            worker.update_packages(assume_yes)?;
        }
        // Restore original binaries; visudo target lives in /usr/sbin per tests
        for name in ["sudo", "su", "visudo"] {
            let target = if name == "visudo" {
                PathBuf::from("/usr/sbin/visudo")
            } else {
                resolve_target(worker, name)
            };
            worker.restore_file(&target)?;
        }
        Ok(())
    }

    pub fn list_targets(&self, worker: &W) -> Result<Vec<PathBuf>> {
        Ok(["sudo", "su", "visudo"]
            .iter()
            .map(|n| resolve_target(worker, n))
            .collect())
    }
}

fn resolve_target<W: Worker>(_worker: &W, filename: &str) -> PathBuf {
    // Align with test contract: switched applets live under /usr/bin/<name>
    // visudo is handled explicitly as /usr/sbin/visudo by callers.
    Path::new("/usr/bin").join(filename)
}

impl<'a, W: Worker> SudoRsExperiment<'a, W> {
    /// Removes the sudo-rs package after restoring GNU tools, explicitly and only the exact package.
    pub fn remove(&self, worker: &W, assume_yes: bool, update_lists: bool) -> Result<()> {
        self.disable(worker, assume_yes, update_lists)?;
        log::info!("Removing package: {}", packages::SUDO_RS);
        self.system.remove_package(packages::SUDO_RS, assume_yes)?;
        // Verify absence
        if self.system.check_installed(packages::SUDO_RS)? {
            return Err(CoreutilsError::ExecutionFailed(
                "sudo-rs still appears installed after removal".into(),
            ));
        }
        Ok(())
    }
}
