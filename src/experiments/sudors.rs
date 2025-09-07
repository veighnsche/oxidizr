use crate::error::{CoreutilsError, Result};
use crate::utils::Distribution;
use crate::utils::worker::Worker;
use std::path::{Path, PathBuf};

pub struct SudoRsExperiment<'a> {
    pub system: &'a dyn Worker,
}

// Prefer cargo-style install location, but accept Arch packaging under /usr/bin/*-rs.
fn find_sudors_source(name: &str) -> Option<PathBuf> {
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

impl<'a> SudoRsExperiment<'a> {
    pub fn name(&self) -> &'static str {
        "sudo-rs"
    }

    pub fn supported_releases(&self) -> Vec<String> {
        vec!["rolling".into()]
    }

    pub fn check_compatible<W: Worker>(&self, worker: &W) -> Result<bool> {
        let d: Distribution = worker.distribution()?;
        Ok(d.id.eq_ignore_ascii_case("arch")
            && self.supported_releases().iter().any(|r| r == &d.release))
    }

    pub fn enable<W: Worker>(
        &self,
        worker: &W,
        _assume_yes: bool,
        update_lists: bool,
    ) -> Result<()> {
        if !self.check_compatible(worker)? {
            return Err(CoreutilsError::Incompatible(
                "Unsupported Arch release".into(),
            ));
        }
        if update_lists {
            worker.update_packages()?;
        }
        // Install sudo-rs
        worker.install_package("sudo-rs")?;
        // Replace sudo, su, visudo with binaries provided by sudo-rs. The Arch package layout may
        // install either into /usr/lib/cargo/bin/<name> or as /usr/bin/<name>-rs. Detect robustly.
        for (name, target) in [
            ("sudo", resolve_target(worker, "sudo")),
            ("su", resolve_target(worker, "su")),
            ("visudo", PathBuf::from("/usr/sbin/visudo")),
        ] {
            log::info!("Preparing sudo-rs applet '{}'", name);
            let source = find_sudors_source(name);
            let source = source.ok_or_else(|| {
                CoreutilsError::ExecutionFailed(format!(
                    "Could not find installed sudo-rs binary for '{0}'. \
                 Checked: /usr/lib/cargo/bin/{0} and /usr/bin/{0}-rs. \
                 Hints: ensure 'sudo-rs' is installed and provides '{0}' on this distro.",
                    name
                ))
            })?;
            log::info!(
                "Linking sudo-rs '{}' from {} -> {}",
                name,
                source.display(),
                target.display()
            );
            worker.replace_file_with_symlink(&source, &target)?;
        }
        Ok(())
    }

    pub fn disable<W: Worker>(&self, worker: &W, update_lists: bool) -> Result<()> {
        if update_lists {
            worker.update_packages()?;
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
        worker.remove_package("sudo-rs")?;
        Ok(())
    }

    pub fn list_targets<W: Worker>(&self, worker: &W) -> Result<Vec<PathBuf>> {
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
