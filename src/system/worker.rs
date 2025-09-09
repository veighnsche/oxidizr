use crate::checks::Distribution;
use crate::error::{Error, Result};
use crate::logging::audit_event;
use crate::symlink;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};
use which::which;

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

    /// Get current distribution information
    pub fn distribution(&self) -> Result<Distribution> {
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

    /// Check if official repositories (e.g., [extra]) are available
    pub fn extra_repo_available(&self) -> Result<bool> {
        // 1) Prefer a concise repo list to avoid false positives (e.g., NoExtract)
        if let Ok(out) = {
            tracing::debug!(cmd = %"pacman-conf --repo-list", "exec");
            let o = std::process::Command::new("pacman-conf")
                .args(["--repo-list"]) // lists repo names, one per line
                .output();
            if let Ok(ref o2) = o {
                tracing::debug!(status = ?o2.status.code(), "exit");
            }
            o
        } {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if out.status.success() {
                let found = stdout
                    .lines()
                    .map(|s| s.trim().to_ascii_lowercase())
                    .any(|l| l == "extra");
                let _ = audit_event(
                    "worker",
                    "extra_repo_available.repo_list",
                    if found { "detected" } else { "not_detected" },
                    "pacman-conf --repo-list",
                    &stdout,
                    out.status.code(),
                );
                if found {
                    return Ok(true);
                }
                // Do not return false yet; fall through to additional heuristics
            }
        }

        // 2) Probe pacman sync DB for the 'extra' repo directly. Requires that callers refreshed (-Sy).
        if let Ok(status) = {
            tracing::debug!(cmd = %"pacman -Sl extra", "exec");
            let s = std::process::Command::new("pacman")
                .args(["-Sl", "extra"]) // list packages in 'extra'
                .status();
            if let Ok(ref st) = s {
                tracing::debug!(status = ?st.code(), "exit");
            }
            s
        } {
            let _ = audit_event(
                "worker",
                "extra_repo_available.pacman_sl",
                if status.success() { "detected" } else { "not_detected" },
                "pacman -Sl extra",
                "",
                status.code(),
            );
            if status.success() {
                return Ok(true);
            }
        }

        // 3) Fallback to full config dump; look for an explicit [extra] section
        if let Ok(out) = {
            tracing::debug!(cmd = %"pacman-conf -l", "exec");
            let o = std::process::Command::new("pacman-conf")
                .args(["-l"]) // list configuration
                .output();
            if let Ok(ref o2) = o {
                tracing::debug!(status = ?o2.status.code(), "exit");
            }
            o
        } {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if out.status.success() {
                let found = stdout.to_ascii_lowercase().contains("[extra]");
                let _ = audit_event(
                    "worker",
                    "extra_repo_available.conf_dump",
                    if found { "detected" } else { "not_detected" },
                    "pacman-conf -l",
                    &stdout,
                    out.status.code(),
                );
                if found {
                    return Ok(true);
                }
            }
        }

        // 4) Last resort: parse /etc/pacman.conf for a [extra] section
        let conf = fs::read_to_string("/etc/pacman.conf").unwrap_or_default();
        let found = conf.to_ascii_lowercase().contains("[extra]");
        let _ = audit_event(
            "worker",
            "extra_repo_available.file_fallback",
            if found { "detected" } else { "not_detected" },
            "/etc/pacman.conf",
            "",
            None,
        );
        Ok(found)
    }

    /// Get available AUR helper name if any
    pub fn aur_helper_name(&self) -> Result<Option<String>> {
        let candidates = self.aur_helper_candidates();
        for h in &candidates {
            if which(h).is_ok() {
                let _ = audit_event(
                    "worker",
                    "aur_helper_name",
                    "found",
                    h,
                    "",
                    None,
                );
                return Ok(Some(h.to_string()));
            }
        }
        let _ = audit_event(
            "worker",
            "aur_helper_name",
            "not_found",
            &self.aur_helper,
            "",
            None,
        );
        Ok(None)
    }

    /// Check if a package exists in official repos (pacman -Si)
    pub fn repo_has_package(&self, package: &str) -> Result<bool> {
        if !Self::is_valid_package_name(package) {
            return Ok(false);
        }
        tracing::debug!(cmd = %format!("pacman -Si {}", package), "exec");
        let status = std::process::Command::new("pacman")
            .args(["-Si", package])
            .status()?;
        tracing::debug!(status = ?status.code(), "exit");
        let _ = audit_event(
            "worker",
            "repo_has_package",
            if status.success() { "yes" } else { "no" },
            &format!("pacman -Si {}", package),
            "",
            status.code(),
        );
        Ok(status.success())
    }

    /// Update package databases (pacman -Sy)
    pub fn update_packages(&self, assume_yes: bool) -> Result<()> {
        if self.dry_run {
            tracing::info!("[dry-run] pacman -Sy");
            return Ok(());
        }
        
        if !self.wait_for_pacman_lock_clear()? {
            return Err(Error::ExecutionFailed(
                "pacman database lock present at /var/lib/pacman/db.lck; retry later".into(),
            ));
        }
        
        let mut args = vec!["-Sy"];
        if assume_yes {
            args.push("--noconfirm");
        }
        
        tracing::debug!(cmd = %format!("pacman {}", args.join(" ")), "exec");
        let status = std::process::Command::new("pacman").args(&args).status()?;
        tracing::debug!(status = ?status.code(), "exit");
        let _ = audit_event(
            "worker",
            "update_packages",
            if status.success() { "ok" } else { "error" },
            &format!("pacman {}", args.join(" ")),
            "",
            status.code(),
        );
        
        if status.success() {
            Ok(())
        } else {
            Err(Error::ExecutionFailed(
                "pacman -Sy failed (could not refresh package databases)".into(),
            ))
        }
    }

    /// Check if a package is installed
    pub fn check_installed(&self, package: &str) -> Result<bool> {
        let status = std::process::Command::new("pacman")
            .args(["-Qi", package])
            .status()?;
        let _ = audit_event(
            "worker",
            "check_installed",
            if status.success() { "present" } else { "absent" },
            &format!("pacman -Qi {}", package),
            "",
            status.code(),
        );
        Ok(status.success())
    }

    /// Install a package with policy enforcement
    pub fn install_package(&self, package: &str, assume_yes: bool) -> Result<()> {
        if !Self::is_valid_package_name(package) {
            return Err(Error::ExecutionFailed(format!(
                "Invalid or unsafe package name: {}",
                package
            )));
        }
        
        if self.dry_run {
            tracing::info!("[dry-run] pacman -S --noconfirm {}", package);
            return Ok(());
        }
        
        // If already installed, do nothing
        if self.check_installed(package)? {
            tracing::info!("Package '{}' already installed (skipping)", package);
            tracing::info!("✅ Expected: '{}' installed, Received: present", package);
            return Ok(());
        }
        
        if !self.wait_for_pacman_lock_clear()? {
            return Err(Error::ExecutionFailed(
                "pacman database lock present at /var/lib/pacman/db.lck; retry later".into(),
            ));
        }
        
        // Try pacman first, unless we know this is an AUR-only package not present in official repos
        let mut attempted_pacman = false;
        let mut pacman_status_ok = false;
        if package != "uutils-findutils-bin" || self.repo_has_package(package).unwrap_or(false) {
            let mut args = vec!["-S"]; 
            if assume_yes {
                args.push("--noconfirm");
            }
            args.push(package);

            tracing::debug!(cmd = %format!("pacman {}", args.join(" ")), "exec");
            let pacman_status = std::process::Command::new("pacman").args(&args).status()?;
            tracing::debug!(status = ?pacman_status.code(), "exit");
            attempted_pacman = true;
            pacman_status_ok = pacman_status.success();
            let _ = audit_event(
                "worker",
                "install_package.pacman",
                if pacman_status.success() { "ok" } else { "failed_or_unavailable" },
                &format!("pacman {}", args.join(" ")),
                "",
                pacman_status.code(),
            );
            
            if pacman_status_ok && self.check_installed(package)? {
                tracing::info!("✅ Expected: '{}' installed, Received: present", package);
                return Ok(());
            }
        } else {
            // Explicitly record that we skipped pacman because the package is not in official repos
            let _ = audit_event(
                "worker",
                "install_package.pacman",
                "skipped_official_absent",
                package,
                "uutils-findutils-bin is AUR-only; pacman -Si indicates absent",
                None,
            );
        }
        
        // Selective policy: allow AUR fallback only for uutils-findutils-bin
        if package == "uutils-findutils-bin" {
            let candidates = self.aur_helper_candidates();
            let available_iter = candidates.into_iter().filter(|h| which(h).is_ok());
            let mut tried_any = false;
            
            for h in available_iter {
                let mut aur_cmd_str = h.to_string();
                if assume_yes {
                    // Batch install must come before the operation for paru
                    aur_cmd_str.push_str(" --batchinstall --noconfirm");
                }
                aur_cmd_str.push_str(" -S --needed");
                aur_cmd_str.push_str(&format!(" {}", package));

                tracing::info!("Running AUR helper: su - builder -c '{}'", aur_cmd_str);
                let aur_status = std::process::Command::new("su")
                    .args(["-", "builder", "-c", &aur_cmd_str])
                    .status()?;
                tracing::debug!(status = ?aur_status.code(), "exit");
                    
                let _ = audit_event(
                    "worker",
                    "install_package.aur",
                    if aur_status.success() { "ok" } else { "error" },
                    &format!("su - builder -c '{}'", aur_cmd_str),
                    &format!("helper={}", h),
                    aur_status.code(),
                );

                if aur_status.success() && self.check_installed(package)? {
                    return Ok(());
                }
                tried_any = true;
            }
            
            if !tried_any {
                return Err(Error::ExecutionFailed(format!(
                    "❌ Expected: '{}' installed, Received: absent. Reason: no AUR helper found. Install an AUR helper (e.g., paru or yay) or pass --package-manager to specify one.",
                    package
                )));
            }
            
            return Err(Error::ExecutionFailed(format!(
                "❌ Expected: '{}' installed, Received: absent. {} Failed to install via pacman{} or any available AUR helper. Ensure networking and helper are functional.",
                package,
                if attempted_pacman { "" } else { "(official repos do not carry this package)." },
                if attempted_pacman && !pacman_status_ok { " (pacman reported target not found)" } else { "" }
            )));
        }
        
        // Official-only policy for all other packages
        Err(Error::ExecutionFailed(format!(
            "❌ Expected: '{}' installed, Received: absent. Failed to install from official repositories (pacman -S). Package may be unavailable in configured repos or mirrors.",
            package
        )))
    }

    /// Remove a package (explicit names only, no wildcards)
    pub fn remove_package(&self, package: &str, assume_yes: bool) -> Result<()> {
        if !Self::is_valid_package_name(package) {
            return Err(Error::ExecutionFailed(format!(
                "Invalid or unsafe package name for removal: {}",
                package
            )));
        }
        
        if self.dry_run {
            tracing::info!("[dry-run] pacman -R --noconfirm {}", package);
            return Ok(());
        }

        // If not installed, do nothing
        if !self.check_installed(package)? {
            tracing::info!("Package '{}' not installed, skipping removal", package);
            tracing::info!("✅ Expected: '{}' absent, Received: absent", package);
            return Ok(());
        }
        
        if !self.wait_for_pacman_lock_clear()? {
            return Err(Error::ExecutionFailed(
                "pacman database lock present at /var/lib/pacman/db.lck; retry later".into(),
            ));
        }
        
        let mut args = vec!["-R"];
        if assume_yes {
            args.push("--noconfirm");
        }
        args.push(package);
        
        tracing::debug!(cmd = %format!("pacman {}", args.join(" ")), "exec");
        let status = std::process::Command::new("pacman").args(&args).status()?;
        tracing::debug!(status = ?status.code(), "exit");
        let _ = audit_event(
            "worker",
            "remove_package",
            if status.success() { "ok" } else { "error" },
            &format!("pacman {}", args.join(" ")),
            "",
            status.code(),
        );
        
        if status.success() {
            // Verify absence after removal for clarity
            if self.check_installed(package)? {
                tracing::error!("❌ Expected: '{}' absent after removal, Received: present", package);
                return Err(Error::ExecutionFailed(format!(
                    "❌ Expected: '{}' absent after removal, Received: present",
                    package
                )));
            }
            tracing::info!("✅ Expected: '{}' absent after removal, Received: absent", package);
            Ok(())
        } else {
            Err(Error::ExecutionFailed(format!(
                "❌ Expected: '{}' absent after removal, Received: present (pacman -R failed)",
                package
            )))
        }
    }

    /// Find a binary in PATH
    pub fn which(&self, name: &str) -> Result<Option<PathBuf>> {
        Ok(which::which(name).ok())
    }

    /// Replace file with symlink (delegated to symlink module)
    pub fn replace_file_with_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        symlink::replace_file_with_symlink(source, target, self.dry_run)
    }

    /// Restore file from backup (delegated to symlink module)  
    pub fn restore_file(&self, target: &Path) -> Result<()> {
        symlink::restore_file(target, self.dry_run)
    }

    // Private helper methods
    
    fn wait_for_pacman_lock_clear(&self) -> Result<bool> {
        if !Path::new(PACMAN_LOCK).exists() {
            return Ok(true);
        }
        
        match self.wait_lock_secs {
            None => Ok(false),
            Some(secs) => {
                let start = Instant::now();
                let timeout = Duration::from_secs(secs);
                while start.elapsed() < timeout {
                    if !Path::new(PACMAN_LOCK).exists() {
                        return Ok(true);
                    }
                    sleep(PACMAN_LOCK_CHECK_INTERVAL);
                }
                Ok(!Path::new(PACMAN_LOCK).exists())
            }
        }
    }

    fn aur_helper_candidates(&self) -> Vec<&str> {
        if !self.aur_helper.is_empty() && self.aur_helper != "auto" && self.aur_helper != "none" {
            vec![self.aur_helper.as_str(), "paru", "yay", "trizen", "pamac"]
        } else {
            vec!["paru", "yay", "trizen", "pamac"]
        }
    }

    fn is_valid_package_name(name: &str) -> bool {
        // Package names should only contain alphanumeric, dash, underscore, plus, and dot
        // and should not start with dash
        if name.is_empty() || name.starts_with('-') {
            return false;
        }
        name.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '+' || c == '.')
    }
}
