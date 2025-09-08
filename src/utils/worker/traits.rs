use crate::error::Result;
use crate::utils::Distribution;
use std::path::{Path, PathBuf};

// Abstracts system operations needed by experiments.
pub trait Worker {
    // Distro/release
    fn distribution(&self) -> Result<Distribution>;

    // Package management
    fn update_packages(&self, assume_yes: bool) -> Result<()>;
    fn install_package(&self, package: &str, assume_yes: bool) -> Result<()>;
    fn remove_package(&self, package: &str, assume_yes: bool) -> Result<()>;
    fn check_installed(&self, package: &str) -> Result<bool>;

    // Repository/helper capabilities
    /// True if the official repositories (e.g., [extra]) are available for installations
    fn extra_repo_available(&self) -> Result<bool>;
    /// Returns the detected AUR helper name if one is available (paru, yay, etc.)
    fn aur_helper_name(&self) -> Result<Option<String>>;
    /// Returns true if the official repos can see the given package (pacman -Si)
    fn repo_has_package(&self, package: &str) -> Result<bool>;

    // Filesystem and process helpers
    fn which(&self, name: &str) -> Result<Option<PathBuf>>;
    fn list_files(&self, dir: &Path) -> Result<Vec<PathBuf>>;

    // Symlink management with safety
    fn replace_file_with_symlink(&self, source: &Path, target: &Path) -> Result<()>;
    fn restore_file(&self, target: &Path) -> Result<()>;
}
