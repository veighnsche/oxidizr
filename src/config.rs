/// Configuration constants for the oxidizr-arch system
pub mod paths {
    pub const SYSTEM_BIN_DIR: &str = "/usr/bin";
    pub const SYSTEM_SBIN_DIR: &str = "/usr/sbin";
    pub const SYSTEM_LIB_DIR: &str = "/usr/lib";
    pub const CARGO_BIN_DIR: &str = "/usr/lib/cargo/bin";
    pub const UUTILS_LIB_DIR: &str = "/usr/lib/uutils";
    pub const PACMAN_LOCK: &str = "/var/lib/pacman/db.lck";
    pub const BACKUP_SUFFIX: &str = ".oxidizr.bak";
}

pub mod packages {
    pub const UUTILS_COREUTILS: &str = "uutils-coreutils";
    // Use the binary AUR package to avoid provider prompts during non-interactive installations
    pub const UUTILS_FINDUTILS: &str = "uutils-findutils-bin";
    pub const SUDO_RS: &str = "sudo-rs";
}

pub mod aur_helpers {
    pub const PARU: &str = "paru";
    pub const YAY: &str = "yay";
    pub const TRIZEN: &str = "trizen";
    pub const PAMAC: &str = "pamac";

    pub const DEFAULT_HELPERS: [&str; 4] = [PARU, YAY, TRIZEN, PAMAC];
}

pub mod security {
    pub const MAX_PATH_LENGTH: usize = 4096;
    pub const MAX_PACKAGE_NAME_LENGTH: usize = 256;
    pub const AUDIT_LOG_SYSTEM: &str = "/var/log/oxidizr-arch-audit.log";
    pub const AUDIT_LOG_USER: &str = ".oxidizr-arch-audit.log";
}

pub mod timeouts {
    use std::time::Duration;

    pub const PACMAN_LOCK_CHECK_INTERVAL: Duration = Duration::from_millis(500);
    pub const DEFAULT_WAIT_LOCK_SECS: u64 = 30;
    pub const DOCKER_RUN_TIMEOUT: Duration = Duration::from_secs(1800); // 30 minutes
}
