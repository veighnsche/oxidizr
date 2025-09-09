use std::time::Duration;
use std::path::PathBuf;

const PACMAN_LOCK: &str = "/var/lib/pacman/db.lck";
const PACMAN_LOCK_CHECK_INTERVAL: Duration = Duration::from_millis(500);

/// System worker for all OS operations
pub struct Worker {
    pub aur_helper: String,
    pub aur_user: Option<String>,
    pub dry_run: bool,
    pub wait_lock_secs: Option<u64>,
    pub package_override: Option<String>,
    pub bin_dir_override: Option<PathBuf>,
    pub unified_binary_override: Option<PathBuf>,
    pub force_restore_best_effort: bool,
}

impl Worker {
    /// Create a new worker
    pub fn new(
        aur_helper: String,
        aur_user: Option<String>,
        dry_run: bool,
        wait_lock_secs: Option<u64>,
        package_override: Option<String>,
        bin_dir_override: Option<PathBuf>,
        unified_binary_override: Option<PathBuf>,
        force_restore_best_effort: bool,
    ) -> Self {
        Self {
            aur_helper,
            aur_user,
            dry_run,
            wait_lock_secs,
            package_override,
            bin_dir_override,
            unified_binary_override,
            force_restore_best_effort,
        }
    }
}

// Submodules providing the rest of the Worker methods
mod distro;
mod aur;
mod packages;
mod fs_ops;
