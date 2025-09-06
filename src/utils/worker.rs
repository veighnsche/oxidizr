use crate::error::{CoreutilsError, Result};
use crate::utils::Distribution;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use which::which;
use std::thread::sleep;
use std::time::{Duration, Instant};

// Abstracts system operations needed by experiments.
pub trait Worker {
    // Distro/release
    fn distribution(&self) -> Result<Distribution>;

    // Package management
    fn update_packages(&self) -> Result<()>;
    fn install_package(&self, package: &str) -> Result<()>;
    fn remove_package(&self, package: &str) -> Result<()>;
    fn check_installed(&self, package: &str) -> Result<bool>;

    // Filesystem and process helpers
    fn which(&self, name: &str) -> Result<Option<PathBuf>>;
    fn list_files(&self, dir: &Path) -> Result<Vec<PathBuf>>;

    // Symlink management with safety
    fn replace_file_with_symlink(&self, source: &Path, target: &Path) -> Result<()>;
    fn restore_file(&self, target: &Path) -> Result<()>;
}

impl System {
    fn wait_for_pacman_lock_clear(&self) -> Result<bool> {
        if !pacman_locked() { return Ok(true); }
        match self.wait_lock_secs {
            None => Ok(false),
            Some(secs) => {
                let start = Instant::now();
                let timeout = Duration::from_secs(secs);
                while start.elapsed() < timeout {
                    if !pacman_locked() { return Ok(true); }
                    sleep(Duration::from_millis(500));
                }
                Ok(!pacman_locked())
            }
        }
    }
}

// A system implementation for Arch-like systems, with dry-run support.
pub struct System {
    pub aur_helper: String,
    pub dry_run: bool,
    pub wait_lock_secs: Option<u64>,
}

impl Worker for System {
    fn distribution(&self) -> Result<Distribution> {
        // Parse /etc/os-release for ID and ID_LIKE. Normalize Arch-like
        // derivatives (Manjaro, EndeavourOS, Artix) to id = "arch".
        let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
        let mut id: Option<String> = None;
        let mut id_like: Option<String> = None;
        for line in content.lines() {
            if id.is_none() {
                if let Some(rest) = line.strip_prefix("ID=") {
                    id = Some(rest.trim_matches('"').to_string());
                }
            }
            if id_like.is_none() {
                if let Some(rest) = line.strip_prefix("ID_LIKE=") {
                    id_like = Some(rest.trim_matches('"').to_string());
                }
            }
        }
        let id_lower = id.unwrap_or_else(|| "arch".to_string()).to_ascii_lowercase();
        let id_like_lower = id_like.unwrap_or_default().to_ascii_lowercase();
        // Include common Arch derivatives
        let arch_markers = ["arch", "manjaro", "endeavouros", "artix", "garuda", "rebornos", "rebornos", "reborn"];
        let is_arch_like = arch_markers.iter().any(|m| id_lower.contains(m))
            || arch_markers.iter().any(|m| id_like_lower.contains(m));
        let norm_id = if is_arch_like { "arch".to_string() } else { id_lower };
        Ok(Distribution { id: norm_id, release: "rolling".to_string() })
    }

    fn update_packages(&self) -> Result<()> {
        if self.dry_run { log::info!("[dry-run] pacman -Sy"); return Ok(()); }
        if !self.wait_for_pacman_lock_clear()? {
            return Err(CoreutilsError::ExecutionFailed("pacman database lock present at /var/lib/pacman/db.lck; retry later".into()));
        }
        let status = std::process::Command::new("pacman").args(["-Sy"]).status()?;
        if status.success() { Ok(()) } else { Err(CoreutilsError::ExecutionFailed("pacman -Sy failed".into())) }
    }

    fn install_package(&self, package: &str) -> Result<()> {
        if self.dry_run {
            log::info!("[dry-run] pacman -S --noconfirm {}", package);
            log::info!("[dry-run] <aur-helper> -S --noconfirm {} (fallback)", package);
            return Ok(());
        }
        // If already installed, do nothing (avoids invoking AUR helpers as root)
        if self.check_installed(package)? { return Ok(()); }
        if !self.wait_for_pacman_lock_clear()? {
            return Err(CoreutilsError::ExecutionFailed("pacman database lock present at /var/lib/pacman/db.lck; retry later".into()));
        }
        // Try pacman, then fall back to AUR helper(s)
        let status = std::process::Command::new("pacman").args(["-S", "--noconfirm", package]).status()?;
        if status.success() { return Ok(()); }
        // Choose helper: prefer configured, else detect installed
        let candidates = aur_helper_candidates(&self.aur_helper);
        let mut available_iter = candidates.clone().into_iter().filter(|h| which(h).is_ok());
        let mut tried_any = false;
        for h in available_iter.by_ref() {
            let status = std::process::Command::new(h).args(["-S", "--noconfirm", package]).status();
            if let Ok(s) = status { if s.success() { return Ok(()); } }
            tried_any = true;
        }
        if !tried_any {
            return Err(CoreutilsError::ExecutionFailed(format!(
                "No AUR helper found. Tried: {}. Install an AUR helper (e.g., paru or yay) or pass --package-manager to specify one.",
                candidates.join(", ")
            )));
        }
        Err(CoreutilsError::ExecutionFailed(format!("Failed to install {} via pacman or any available AUR helper (checked configured and common helpers)", package)))
    }

    fn remove_package(&self, package: &str) -> Result<()> {
        if self.dry_run { log::info!("[dry-run] pacman -R --noconfirm {}", package); return Ok(()); }
        if !self.wait_for_pacman_lock_clear()? {
            return Err(CoreutilsError::ExecutionFailed("pacman database lock present at /var/lib/pacman/db.lck; retry later".into()));
        }
        let status = std::process::Command::new("pacman").args(["-R", "--noconfirm", package]).status()?;
        if status.success() { Ok(()) } else { Err(CoreutilsError::ExecutionFailed(format!("Failed to remove {}", package))) }
    }

    fn check_installed(&self, package: &str) -> Result<bool> {
        let status = std::process::Command::new("pacman").args(["-Qi", package]).status()?;
        Ok(status.success())
    }

    fn which(&self, name: &str) -> Result<Option<PathBuf>> {
        Ok(which::which(name).ok())
    }

    fn list_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        if !dir.exists() { return Ok(out); }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() { out.push(path); }
        }
        Ok(out)
    }

    fn replace_file_with_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        // Gather initial state for logging/instrumentation
        let existed = target.exists();
        let is_symlink = fs::symlink_metadata(target)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        let current_dest = if is_symlink { fs::read_link(target).ok() } else { None };
        log::info!(
            "replace_file_with_symlink pre-state: target={}, existed={}, is_symlink={}, current_dest={}",
            target.display(),
            existed,
            is_symlink,
            current_dest
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<none>".into())
        );

        if self.dry_run {
            log::info!(
                "[dry-run] would ensure symlink {} -> {} (updating/replacing as needed)",
                source.display(),
                target.display()
            );
            return Ok(());
        }

        // If a symlink already exists, verify it points to the desired source; if not, replace it
        if is_symlink {
            // Try to canonicalize both sides for robust comparison (handles relative symlink targets)
            let desired = fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
            let mut resolved_current = current_dest.clone().unwrap_or_default();
            if resolved_current.is_relative() {
                if let Some(parent) = target.parent() { resolved_current = parent.join(resolved_current); }
            }
            let resolved_current = fs::canonicalize(&resolved_current).unwrap_or(resolved_current);
            if resolved_current == desired {
                log::info!(
                    "Existing symlink already correct: {} -> {} (no action)",
                    target.display(),
                    resolved_current.display()
                );
                return Ok(());
            } else {
                log::info!(
                    "Replacing existing symlink: {} currently -> {}, desired -> {}",
                    target.display(),
                    current_dest
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<unreadable>".into()),
                    source.display()
                );
                fs::remove_file(target)?;
                unix_fs::symlink(source, target)?;
                log::info!(
                    "Symlink updated: {} -> {}",
                    target.display(),
                    source.display()
                );
                return Ok(());
            }
        }

        // For regular files: backup then replace with symlink
        if existed {
            let backup = backup_path(target);
            log::info!("Backing up {} -> {}", target.display(), backup.display());
            fs::copy(target, &backup)?;
            // Reapply permissions from original to backup
            let meta = fs::metadata(target)?;
            let perm = meta.permissions();
            fs::set_permissions(&backup, perm)?;
            fs::remove_file(target)?;
        }
        // Ensure parent exists
        if let Some(parent) = target.parent() { fs::create_dir_all(parent)?; }
        // Remove leftover target then symlink
        let _ = fs::remove_file(target);
        unix_fs::symlink(source, target)?;
        log::info!("Symlink created: {} -> {}", target.display(), source.display());
        Ok(())
    }

    fn restore_file(&self, target: &Path) -> Result<()> {
        let backup = backup_path(target);
        if backup.exists() {
            if self.dry_run {
                log::info!("[dry-run] would restore {} from {}", target.display(), backup.display());
                return Ok(());
            }
            log::info!("Restoring {} <- {}", target.display(), backup.display());
            // Remove symlink or leftover
            let _ = fs::remove_file(target);
            fs::rename(backup, target)?;
        } else {
            log::warn!("No backup for {}, leaving as-is", target.display());
        }
        Ok(())
    }
}

fn backup_path(target: &Path) -> PathBuf {
    let name = target.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{}.oxidizr.bak", name))
}

fn pacman_locked() -> bool {
    Path::new("/var/lib/pacman/db.lck").exists()
}

fn aur_helper_candidates(configured: &str) -> Vec<&str> {
    if !configured.is_empty() {
        vec![configured, "paru", "yay", "trizen", "pamac"]
    } else {
        vec!["paru", "yay", "trizen", "pamac"]
    }
}
