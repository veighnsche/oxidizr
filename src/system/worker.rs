use std::time::Duration;
use std::path::PathBuf;

const PACMAN_LOCK: &str = "/var/lib/pacman/db.lck";
const PACMAN_LOCK_CHECK_INTERVAL: Duration = Duration::from_millis(500);

/// System worker for all OS operations
pub struct Worker {
    pub aur_helper: String,
    pub dry_run: bool,
    pub wait_lock_secs: Option<u64>,
    pub package_override: Option<String>,
    pub bin_dir_override: Option<PathBuf>,
    pub unified_binary_override: Option<PathBuf>,
}

impl Worker {
    /// Create a new worker
    pub fn new(
        aur_helper: String,
        dry_run: bool,
        wait_lock_secs: Option<u64>,
        package_override: Option<String>,
        bin_dir_override: Option<PathBuf>,
        unified_binary_override: Option<PathBuf>,
    ) -> Self {
        Self {
            aur_helper,
            dry_run,
            wait_lock_secs,
            package_override,
            bin_dir_override,
            unified_binary_override,
        }
    }
}

// Submodules providing the rest of the Worker methods
mod distro;
mod aur;
mod packages;
mod fs_ops;
