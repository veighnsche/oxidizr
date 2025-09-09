use std::time::Duration;

const PACMAN_LOCK: &str = "/var/lib/pacman/db.lck";
const PACMAN_LOCK_CHECK_INTERVAL: Duration = Duration::from_millis(500);

/// System worker for all OS operations
pub struct Worker {
    pub aur_helper: String,
    pub dry_run: bool,
    pub wait_lock_secs: Option<u64>,
    pub flip_checksums: bool,
}

impl Worker {
    /// Create a new worker
    pub fn new(aur_helper: String, dry_run: bool, wait_lock_secs: Option<u64>, flip_checksums: bool) -> Self {
        Self {
            aur_helper,
            dry_run,
            wait_lock_secs,
            flip_checksums,
        }
    }
}

// Submodules providing the rest of the Worker methods
mod distro;
mod aur;
mod packages;
mod fs_ops;
