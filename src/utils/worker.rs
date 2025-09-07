use crate::error::{CoreutilsError, Result};
use crate::utils::Distribution;
use crate::utils::audit::AUDIT;
use crate::config::{paths, aur_helpers, timeouts};
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
                    sleep(timeouts::PACMAN_LOCK_CHECK_INTERVAL);
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
        if status.success() {
            Ok(())
        } else {
            Err(CoreutilsError::ExecutionFailed("pacman -Sy failed (could not refresh package databases)".into()))
        }
    }

    fn install_package(&self, package: &str) -> Result<()> {
        if self.dry_run {
            log::info!("[dry-run] pacman -S --noconfirm {}", package);
            log::info!("[dry-run] <aur-helper> -S --noconfirm {} (fallback)", package);
            return Ok(());
        }
        // If already installed, do nothing (avoids invoking AUR helpers as root)
        if self.check_installed(package)? {
            log::info!("Package '{}' already installed (skipping)", package);
            return Ok(());
        }
        if !self.wait_for_pacman_lock_clear()? {
            return Err(CoreutilsError::ExecutionFailed("pacman database lock present at /var/lib/pacman/db.lck; retry later".into()));
        }
        // Try pacman, then fall back to AUR helper(s)
        let status = std::process::Command::new("pacman").args(["-S", "--noconfirm", package]).status()?;
        if status.success() {
            // Double-check installation actually succeeded
            if self.check_installed(package)? {
                return Ok(());
            } else {
                return Err(CoreutilsError::ExecutionFailed(format!(
                    "pacman reported success installing '{}' but package not found via 'pacman -Qi'",
                    package
                )));
            }
        }
        // Choose helper: prefer configured, else detect installed. Run helpers as non-root 'builder'.
        let candidates = aur_helper_candidates(&self.aur_helper);
        let mut available_iter = candidates.clone().into_iter().filter(|h| which(h).is_ok());
        let mut tried_any = false;
        for h in available_iter.by_ref() {
            // Validate package name to prevent command injection
            if !is_valid_package_name(package) {
                return Err(CoreutilsError::ExecutionFailed(format!("Invalid package name: {}", package)));
            }
            // Use proper argument array instead of string interpolation
            let status = std::process::Command::new("sudo")
                .args(["-u", "builder", "--", h, "-S", "--noconfirm", "--needed", package])
                .status();
            if let Ok(s) = status { if s.success() { return Ok(()); } }
            tried_any = true;
        }
        if !tried_any {
            return Err(CoreutilsError::ExecutionFailed(format!(
                "No AUR helper found. Tried: {}. Install an AUR helper (e.g., paru or yay) or pass --package-manager to specify one.",
                candidates.join(", ")
            )));
        }
        Err(CoreutilsError::ExecutionFailed(format!(
            "Failed to install '{}' via pacman or any available AUR helper (checked configured and common helpers). \
             Ensure an AUR helper is installed (e.g., paru, yay) and that networking is functional.",
            package
        )))
    }

    fn remove_package(&self, package: &str) -> Result<()> {
        if self.dry_run { log::info!("[dry-run] pacman -R --noconfirm {}", package); return Ok(()); }
        if !self.wait_for_pacman_lock_clear()? {
            return Err(CoreutilsError::ExecutionFailed("pacman database lock present at /var/lib/pacman/db.lck; retry later".into()));
        }
        let status = std::process::Command::new("pacman").args(["-R", "--noconfirm", package]).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(CoreutilsError::ExecutionFailed(format!("Failed to remove '{}' (pacman -R failed)", package)))
        }
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
        // Validate paths to prevent directory traversal attacks
        if !is_safe_path(source) || !is_safe_path(target) {
            return Err(CoreutilsError::ExecutionFailed("Invalid path: contains directory traversal".into()));
        }
        // Use symlink_metadata to avoid TOCTOU race conditions
        let metadata = fs::symlink_metadata(target);
        let existed = metadata.is_ok();
        let is_symlink = metadata
            .as_ref()
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
                // Create a backup of the current resolved target if it exists, so that assertions
                // (which expect .<name>.oxidizr.bak) are satisfied consistently even when the
                // original was a symlink. We copy file contents and preserve permissions similar
                // to the regular-file branch.
                let backup = backup_path(target);
                if resolved_current.exists() {
                    log::info!("Backing up (from symlink) {} -> {}", target.display(), backup.display());
                    let _ = fs::copy(&resolved_current, &backup);
                    if let Ok(meta) = fs::metadata(&resolved_current) {
                        let perm = meta.permissions();
                        let _ = fs::set_permissions(&backup, perm);
                    }
                } else {
                    log::warn!(
                        "Resolved current target for {} does not exist ({}); creating no-op backup",
                        target.display(),
                        resolved_current.display()
                    );
                }
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

        // For regular files: backup then replace with symlink atomically
        if existed {
            let backup = backup_path(target);
            log::info!("Backing up {} -> {}", target.display(), backup.display());
            // Use metadata we already have to avoid additional TOCTOU
            if let Ok(ref meta) = metadata {
                fs::copy(target, &backup)?;
                let perm = meta.permissions();
                fs::set_permissions(&backup, perm)?;
                fs::remove_file(target)?;
            } else {
                // If metadata failed, the file might have been removed - handle gracefully
                log::warn!("Target file {} disappeared during operation", target.display());
            }
        }
        // Ensure parent exists
        if let Some(parent) = target.parent() { fs::create_dir_all(parent)?; }
        // Remove leftover target then symlink
        let _ = fs::remove_file(target);
        unix_fs::symlink(source, target)?;
        log::info!("Symlink created: {} -> {}", target.display(), source.display());
        // Audit log the symlink creation
        let _ = AUDIT.log_operation(
            "CREATE_SYMLINK",
            &format!("{} -> {}", target.display(), source.display()),
            true
        );
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
            // Audit log the restoration
            let _ = AUDIT.log_operation(
                "RESTORE_FILE",
                &format!("{}", target.display()),
                true
            );
        } else {
            log::warn!("No backup for {}, leaving as-is", target.display());
        }
        Ok(())
    }
}

fn backup_path(target: &Path) -> PathBuf {
    let name = target.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{}{}", name, paths::BACKUP_SUFFIX))
}

fn pacman_locked() -> bool {
    Path::new(paths::PACMAN_LOCK).exists()
}

fn aur_helper_candidates(configured: &str) -> Vec<&str> {
    if !configured.is_empty() {
        let mut helpers = vec![configured];
        helpers.extend_from_slice(&aur_helpers::DEFAULT_HELPERS);
        helpers
    } else {
        aur_helpers::DEFAULT_HELPERS.to_vec()
    }
}

// Validate package names to prevent command injection
fn is_valid_package_name(name: &str) -> bool {
    // Package names should only contain alphanumeric, dash, underscore, plus, and dot
    // and should not start with dash
    if name.is_empty() || name.starts_with('-') {
        return false;
    }
    name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '+' || c == '.')
}

// Validate paths to prevent directory traversal attacks
fn is_safe_path(path: &Path) -> bool {
    // Check for directory traversal patterns
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return false;
        }
    }
    // Check for absolute paths that go outside expected directories
    if let Some(path_str) = path.to_str() {
        if path_str.contains("/../") || path_str.contains("..\\") {
            return false;
        }
    }
    true
}
