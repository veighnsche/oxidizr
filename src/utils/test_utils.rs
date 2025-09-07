use crate::error::Result;
use crate::utils::Distribution;
use crate::utils::worker::Worker;
use std::path::{Path, PathBuf};

// A mock worker for testing purposes.
pub struct MockWorker {
    pub distribution: Distribution,
}

impl Default for MockWorker {
    fn default() -> Self {
        Self {
            distribution: Distribution {
                id: "arch".to_string(),
                id_like: "arch".to_string(),
                release: "rolling".to_string(),
            },
        }
    }
}

impl Worker for MockWorker {
    fn distribution(&self) -> Result<Distribution> {
        Ok(self.distribution.clone())
    }

    fn update_packages(&self, _assume_yes: bool) -> Result<()> {
        Ok(())
    }

    fn install_package(&self, _package: &str, _assume_yes: bool) -> Result<()> {
        Ok(())
    }

    fn remove_package(&self, _package: &str, _assume_yes: bool) -> Result<()> {
        Ok(())
    }

    fn check_installed(&self, _package: &str) -> Result<bool> {
        Ok(true)
    }

    fn which(&self, name: &str) -> Result<Option<PathBuf>> {
        Ok(Some(PathBuf::from(format!("/usr/bin/{}", name))))
    }

    fn list_files(&self, _dir: &Path) -> Result<Vec<PathBuf>> {
        Ok(vec![])
    }

    fn replace_file_with_symlink(&self, _source: &Path, _target: &Path) -> Result<()> {
        Ok(())
    }

    fn restore_file(&self, _target: &Path) -> Result<()> {
        Ok(())
    }
}
