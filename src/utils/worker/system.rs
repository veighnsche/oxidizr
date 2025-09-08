use crate::config::timeouts;
use crate::error::{CoreutilsError, Result};
use crate::utils::audit::AUDIT;
use crate::utils::Distribution;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};
use which::which;

use super::helpers::{
    aur_helper_candidates, backup_path, is_safe_path, is_valid_package_name, pacman_locked,
};
use super::traits::Worker;

impl System {
    fn wait_for_pacman_lock_clear(&self) -> Result<bool> {
        if !pacman_locked() {
            return Ok(true);
        }
        match self.wait_lock_secs {
            None => Ok(false),
            Some(secs) => {
                let start = Instant::now();
                let timeout = Duration::from_secs(secs);
                while start.elapsed() < timeout {
                    if !pacman_locked() {
                        return Ok(true);
                    }
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
        // Parse /etc/os-release for ID and ID_LIKE.
        let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
        let mut id: Option<String> = None;
        let mut id_like: Option<String> = None;
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("ID=") {
                id = Some(rest.trim_matches('"').to_string());
            }
            if let Some(rest) = line.strip_prefix("ID_LIKE=") {
                id_like = Some(rest.trim_matches('"').to_string());
            }
        }
        Ok(Distribution {
            id: id.unwrap_or_else(|| "arch".to_string()),
            id_like: id_like.unwrap_or_default(),
            release: "rolling".to_string(),
        })
    }

    fn update_packages(&self, assume_yes: bool) -> Result<()> {
        if self.dry_run {
            log::info!("[dry-run] pacman -Sy");
            return Ok(());
        }
        if !self.wait_for_pacman_lock_clear()? {
            return Err(CoreutilsError::ExecutionFailed(
                "pacman database lock present at /var/lib/pacman/db.lck; retry later".into(),
            ));
        }
        let mut args = vec!["-Sy"];
        if assume_yes {
            args.push("--noconfirm");
        }
        let status = std::process::Command::new("pacman").args(&args).status()?;
        let _ = AUDIT.log_provenance(
            "worker.system",
            "update_packages",
            if status.success() { "ok" } else { "error" },
            &format!("pacman {}", args.join(" ")),
            "",
            status.code(),
        );
        if status.success() {
            Ok(())
        } else {
            Err(CoreutilsError::ExecutionFailed(
                "pacman -Sy failed (could not refresh package databases)".into(),
            ))
        }
    }

    fn install_package(&self, package: &str, assume_yes: bool) -> Result<()> {
        if !is_valid_package_name(package) {
            return Err(CoreutilsError::ExecutionFailed(format!(
                "Invalid or unsafe package name: {}",
                package
            )));
        }
        if self.dry_run {
            log::info!("[dry-run] pacman -S --noconfirm {}", package);
            return Ok(());
        }
        // If already installed, do nothing.
        // This ensures we DO NOT re-install pre-existing packages (e.g., user preinstalled
        // uutils-coreutils or sudo-rs). Experiments will proceed to switch symlinks to the
        // already-installed provider without reinstallation.
        if self.check_installed(package)? {
            log::info!("Package '{}' already installed (skipping)", package);
            return Ok(());
        }
        if !self.wait_for_pacman_lock_clear()? {
            return Err(CoreutilsError::ExecutionFailed(
                "pacman database lock present at /var/lib/pacman/db.lck; retry later".into(),
            ));
        }
        // Try pacman, then fall back to AUR helper(s)
        let mut args = vec!["-S"];
        if assume_yes {
            args.push("--noconfirm");
        }
        args.push(package);

        // Try to install with pacman. If it succeeds, we're done.
        let pacman_status = std::process::Command::new("pacman").args(&args).status()?;
        let _ = AUDIT.log_provenance(
            "worker.system",
            "install_package.pacman",
            if pacman_status.success() { "ok" } else { "failed_or_unavailable" },
            &format!("pacman {}", args.join(" ")),
            "",
            pacman_status.code(),
        );
        if pacman_status.success() && self.check_installed(package)? {
            return Ok(());
        }
        // Selective policy: allow AUR fallback only for packages explicitly permitted.
        if package == "uutils-findutils-bin" {
            // Choose helper: prefer configured, else detect installed. Run helpers as non-root 'builder'.
            let candidates = aur_helper_candidates(&self.aur_helper);
            let mut available_iter = candidates.clone().into_iter().filter(|h| which(h).is_ok());
            let mut tried_any = false;
            for h in available_iter.by_ref() {
                let mut aur_cmd_str = h.to_string();
                if assume_yes {
                    // Batch install must come before the operation for paru
                    aur_cmd_str.push_str(" --batchinstall --noconfirm");
                }
                aur_cmd_str.push_str(" -S --needed");
                aur_cmd_str.push_str(&format!(" {}", package));

                log::info!("Running AUR helper: su - builder -c '{}'", aur_cmd_str);
                let aur_status = std::process::Command::new("su")
                    .args(["-", "builder", "-c", &aur_cmd_str])
                    .status()?;
                let _ = AUDIT.log_provenance(
                    "worker.system",
                    "install_package.aur",
                    if aur_status.success() { "ok" } else { "error" },
                    &format!("su - builder -c '{}'", aur_cmd_str),
                    &format!("helper={}", h),
                    aur_status.code(),
                );

                if aur_status.success() {
                    if self.check_installed(package)? {
                        return Ok(());
                    }
                }
                tried_any = true;
            }
            if !tried_any {
                return Err(CoreutilsError::ExecutionFailed(format!(
                    "No AUR helper found. Tried: {}. Install an AUR helper (e.g., paru or yay) or pass --package-manager to specify one.",
                    candidates.join(", ")
                )));
            }
            return Err(CoreutilsError::ExecutionFailed(format!(
                "Failed to install '{}' via pacman or any available AUR helper (checked configured and common helpers). Ensure networking and helper are functional.",
                package
            )));
        }
        // Official-only policy for all other packages: do not attempt AUR fallback.
        Err(CoreutilsError::ExecutionFailed(format!(
            "Failed to install '{}' from official repositories (pacman -S). Package may be unavailable in configured repos or mirrors.",
            package
        )))
    }

    fn remove_package(&self, package: &str, assume_yes: bool) -> Result<()> {
        // NOTE: Experiments may call this during `disable` (e.g., uutils-* and sudo-rs).
        // Current semantics: if the package is installed, we uninstall it regardless of
        // whether it was originally installed by the user or by a prior `enable` run.
        // If we want a more conservative behavior (e.g., optional purge flag), wire it at
        // the experiment layer and gate calls into this function accordingly.
        if !is_valid_package_name(package) {
            return Err(CoreutilsError::ExecutionFailed(format!(
                "Invalid or unsafe package name for removal: {}",
                package
            )));
        }
        if self.dry_run {
            log::info!("[dry-run] pacman -R --noconfirm {}", package);
            return Ok(());
        }

        // If not installed, do nothing.
        if !self.check_installed(package)? {
            log::info!("Package '{}' not installed, skipping removal", package);
            return Ok(());
        }
        if !self.wait_for_pacman_lock_clear()? {
            return Err(CoreutilsError::ExecutionFailed(
                "pacman database lock present at /var/lib/pacman/db.lck; retry later".into(),
            ));
        }
        let mut args = vec!["-R"];
        if assume_yes {
            args.push("--noconfirm");
        }
        args.push(package);
        let status = std::process::Command::new("pacman").args(&args).status()?;
        let _ = AUDIT.log_provenance(
            "worker.system",
            "remove_package",
            if status.success() { "ok" } else { "error" },
            &format!("pacman {}", args.join(" ")),
            "",
            status.code(),
        );
        if status.success() {
            Ok(())
        } else {
            Err(CoreutilsError::ExecutionFailed(format!(
                "Failed to remove '{}' (pacman -R failed)",
                package
            )))
        }
    }

    fn check_installed(&self, package: &str) -> Result<bool> {
        let status = std::process::Command::new("pacman").args(["-Qi", package]).status()?;
        let _ = AUDIT.log_provenance(
            "worker.system",
            "check_installed",
            if status.success() { "present" } else { "absent" },
            &format!("pacman -Qi {}", package),
            "",
            status.code(),
        );
        Ok(status.success())
    }

    fn which(&self, name: &str) -> Result<Option<PathBuf>> {
        Ok(which::which(name).ok())
    }

    fn list_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        if !dir.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                out.push(path);
            }
        }
        Ok(out)
    }

    fn replace_file_with_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        if source == target {
            log::info!("Source and target are the same ({}), skipping symlink.", source.display());
            return Ok(());
        }
        // Validate paths to prevent directory traversal attacks
        if !is_safe_path(source) || !is_safe_path(target) {
            return Err(CoreutilsError::ExecutionFailed(
                "Invalid path: contains directory traversal".into(),
            ));
        }
        // Use symlink_metadata to avoid TOCTOU race conditions
        let metadata = fs::symlink_metadata(target);
        let existed = metadata.is_ok();
        let is_symlink = metadata
            .as_ref()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        let current_dest = if is_symlink {
            fs::read_link(target).ok()
        } else {
            None
        };
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
            if resolved_current.is_relative()
                && let Some(parent) = target.parent()
            {
                resolved_current = parent.join(resolved_current);
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
                    log::info!(
                        "Backing up (from symlink) {} -> {}",
                        target.display(),
                        backup.display()
                    );
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
                log::warn!(
                    "Target file {} disappeared during operation",
                    target.display()
                );
            }
        }
        // Ensure parent exists
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        // Remove leftover target then symlink
        let _ = fs::remove_file(target);
        unix_fs::symlink(source, target)?;
        log::info!(
            "Symlink created: {} -> {}",
            target.display(),
            source.display()
        );
        // Audit log the symlink creation
        let _ = AUDIT.log_operation(
            "CREATE_SYMLINK",
            &format!("{} -> {}", target.display(), source.display()),
            true,
        );
        Ok(())
    }

    fn restore_file(&self, target: &Path) -> Result<()> {
        // Restores the original file for `target` from its side-by-side backup
        // (.<name>.oxidizr.bak) if present. This undoes the applet symlink installed during
        // `enable` without uninstalling any packages by itself. Package removal (if any)
        // is orchestrated by experiment code.
        let backup = backup_path(target);
        if backup.exists() {
            if self.dry_run {
                log::info!(
                    "[dry-run] would restore {} from {}",
                    target.display(),
                    backup.display()
                );
                return Ok(());
            }
            log::info!("Restoring {} <- {}", target.display(), backup.display());
            // Remove symlink or leftover
            let _ = fs::remove_file(target);
            fs::rename(backup, target)?;
            // Audit log the restoration
            let _ = AUDIT.log_operation("RESTORE_FILE", &format!("{}", target.display()), true);
        } else {
            log::warn!("No backup for {}, leaving as-is", target.display());
        }
        Ok(())
    }

    fn extra_repo_available(&self) -> Result<bool> {
        // Prefer pacman-conf -l, fallback to scanning pacman.conf
        let output = std::process::Command::new("pacman-conf")
            .args(["-l"])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let ok = out.status.success() && stdout.to_ascii_lowercase().contains("[extra]");
            let _ = AUDIT.log_provenance(
                "worker.system",
                "extra_repo_available",
                if ok { "detected" } else { "not_detected" },
                "pacman-conf -l",
                &stdout,
                out.status.code(),
            );
            if out.status.success() {
                return Ok(ok);
            }
        }
        // Fallback: read /etc/pacman.conf
        let conf = fs::read_to_string("/etc/pacman.conf").unwrap_or_default();
        let ok = conf.to_ascii_lowercase().contains("[extra]");
        let _ = AUDIT.log_provenance(
            "worker.system",
            "extra_repo_available.fallback",
            if ok { "detected" } else { "not_detected" },
            "/etc/pacman.conf",
            "",
            None,
        );
        Ok(ok)
    }

    fn aur_helper_name(&self) -> Result<Option<String>> {
        let cands = aur_helper_candidates(&self.aur_helper);
        for h in cands {
            if which(h).is_ok() {
                let _ = AUDIT.log_provenance(
                    "worker.system",
                    "aur_helper_name",
                    "found",
                    h,
                    "",
                    None,
                );
                return Ok(Some(h.to_string()));
            }
        }
        let _ = AUDIT.log_provenance(
            "worker.system",
            "aur_helper_name",
            "not_found",
            &self.aur_helper,
            "",
            None,
        );
        Ok(None)
    }

    fn repo_has_package(&self, package: &str) -> Result<bool> {
        if !is_valid_package_name(package) {
            return Ok(false);
        }
        let status = std::process::Command::new("pacman")
            .args(["-Si", package])
            .status()?;
        let _ = AUDIT.log_provenance(
            "worker.system",
            "repo_has_package",
            if status.success() { "yes" } else { "no" },
            &format!("pacman -Si {}", package),
            "",
            status.code(),
        );
        Ok(status.success())
    }
}
