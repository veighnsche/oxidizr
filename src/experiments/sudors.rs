use crate::error::{CoreutilsError, Result};
use crate::utils::worker::Worker;
use crate::utils::Distribution;
use std::path::{Path, PathBuf};

pub struct SudoRsExperiment<'a> {
    pub system: &'a dyn Worker,
}

impl<'a> SudoRsExperiment<'a> {
    pub fn name(&self) -> &'static str { "sudo-rs" }

    pub fn supported_releases(&self) -> Vec<String> { vec!["rolling".into()] }

    pub fn check_compatible<W: Worker>(&self, worker: &W) -> Result<bool> {
        let d: Distribution = worker.distribution()?;
        Ok(d.id.to_ascii_lowercase() == "arch" && self.supported_releases().iter().any(|r| r == &d.release))
    }

    pub fn enable<W: Worker>(&self, worker: &W, _assume_yes: bool, update_lists: bool) -> Result<()> {
        if !self.check_compatible(worker)? {
            return Err(CoreutilsError::Incompatible("Unsupported Arch release".into()));
        }
        if update_lists { worker.update_packages()?; }
        // Install sudo-rs
        worker.install_package("sudo-rs")?;
        // Replace sudo, su, visudo with sudo-rs binary (scaffold assumption)
        let targets = ["sudo", "su", "visudo"];
        for name in targets {
            let target = resolve_target(worker, name);
            let source = PathBuf::from("/usr/bin/sudo-rs");
            worker.replace_file_with_symlink(&source, &target)?;
        }
        Ok(())
    }

    pub fn disable<W: Worker>(&self, worker: &W, update_lists: bool) -> Result<()> {
        if update_lists { worker.update_packages()?; }
        let targets = ["sudo", "su", "visudo"];
        for name in targets {
            let target = resolve_target(worker, name);
            worker.restore_file(&target)?;
        }
        worker.remove_package("sudo-rs")?;
        Ok(())
    }

    pub fn list_targets<W: Worker>(&self, worker: &W) -> Result<Vec<PathBuf>> {
        Ok(["sudo", "su", "visudo"].iter().map(|n| resolve_target(worker, n)).collect())
    }
}

fn resolve_target<W: Worker>(worker: &W, filename: &str) -> PathBuf {
    if let Ok(Some(path)) = worker.which(filename) {
        path
    } else {
        Path::new("/usr/bin").join(filename)
    }
}
