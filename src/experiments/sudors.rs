use crate::error::{CoreutilsError, Result};
use crate::utils::Distribution;
use crate::utils::worker::Worker;
use std::path::{Path, PathBuf};

pub struct SudoRsExperiment<'a, W: Worker> {
    pub system: &'a W,
    pub package_name: String,
}

// Prefer cargo-style install location, but accept Arch packaging under /usr/bin/*-rs.
fn find_sudors_source<W: Worker>(worker: &W, name: &str) -> Option<PathBuf> {
    // On derivatives, the 'sudo' package is the source of truth.
    if !worker.distribution().map(|d| d.id.eq_ignore_ascii_case("arch")).unwrap_or(false) {
        if let Ok(Some(path)) = worker.which(name) {
            return Some(path);
        }
    }
    let candidates = [
        PathBuf::from(format!("/usr/lib/cargo/bin/{}", name)),
        PathBuf::from(format!("/usr/bin/{}-rs", name)),
    ];
    for c in candidates {
        log::debug!("checking sudo-rs candidate for '{}': {}", name, c.display());
        if Path::new(&c).exists() {
            return Some(c);
        }
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
        // This experiment is only for vanilla Arch, as derivatives have their own sudo.
        Ok(d.id.eq_ignore_ascii_case("arch"))
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
        // Install sudo-rs
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
        if self.package_name == "sudo-rs" {
            self.system.remove_package(&self.package_name, assume_yes)?;
        } else {
            log::info!("Skipping removal of core package: {}", self.package_name);
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
