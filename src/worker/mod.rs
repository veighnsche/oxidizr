use crate::error::{CoreutilsError, Result};
use std::fs;
use std::io;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

/// Abstracts system operations needed by experiments.
pub trait Worker {
    // Distro/release
    fn distribution(&self) -> Result<(String, String)>; // (distro, release)

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

/// A no-op system implementation suitable for scaffolding and tests.
/// Replace methods with real Arch Linux (pacman/AUR helper) interactions later.
pub struct System {
    pub aur_helper: String,
}

#[cfg(not(feature = "arch"))]
impl Worker for System {
    fn distribution(&self) -> Result<(String, String)> {
        Ok(("Arch".to_string(), "rolling".to_string()))
    }
    fn update_packages(&self) -> Result<()> { Ok(()) }
    fn install_package(&self, _package: &str) -> Result<()> { Ok(()) }
    fn remove_package(&self, _package: &str) -> Result<()> { Ok(()) }
    fn check_installed(&self, _package: &str) -> Result<bool> { Ok(false) }
    fn which(&self, _name: &str) -> Result<Option<PathBuf>> { Ok(None) }
    fn list_files(&self, _dir: &Path) -> Result<Vec<PathBuf>> { Ok(vec![]) }
    fn replace_file_with_symlink(&self, _source: &Path, _target: &Path) -> Result<()> { Ok(()) }
    fn restore_file(&self, _target: &Path) -> Result<()> { Ok(()) }
}

#[cfg(feature = "arch")]
impl Worker for System {
    fn distribution(&self) -> Result<(String, String)> {
        // Parse /etc/os-release
        let content = fs::read_to_string("/etc/os-release")?;
        let mut id = String::new();
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("ID=") {
                id = rest.trim_matches('"').to_string();
                break;
            }
        }
        Ok((id.chars().next_uppercase().to_string() + &id.chars().skip(1).collect::<String>(), "rolling".to_string()))
    }

    fn update_packages(&self) -> Result<()> {
        let status = std::process::Command::new("pacman")
            .args(["-Sy"])
            .status()?;
        if status.success() { Ok(()) } else { Err(CoreutilsError::ExecutionFailed("pacman -Sy failed".into())) }
    }

    fn install_package(&self, package: &str) -> Result<()> {
        // Try pacman, then fall back to AUR helper
        let status = std::process::Command::new("pacman")
            .args(["-S", "--noconfirm", package])
            .status()?;
        if status.success() { return Ok(()); }
        let status = std::process::Command::new(&self.aur_helper)
            .args(["-S", "--noconfirm", package])
            .status()?;
        if status.success() { Ok(()) } else { Err(CoreutilsError::ExecutionFailed(format!("Failed to install {}", package))) }
    }

    fn remove_package(&self, package: &str) -> Result<()> {
        let status = std::process::Command::new("pacman")
            .args(["-R", "--noconfirm", package])
            .status()?;
        if status.success() { Ok(()) } else { Err(CoreutilsError::ExecutionFailed(format!("Failed to remove {}", package))) }
    }

    fn check_installed(&self, package: &str) -> Result<bool> {
        let status = std::process::Command::new("pacman")
            .args(["-Qi", package])
            .status()?;
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
        // If target is already a symlink, skip
        if fs::symlink_metadata(target).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
            log::info!("Skipping existing symlink: {}", target.display());
            return Ok(());
        }
        // Backup if exists and not a symlink
        if target.exists() {
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
        Ok(())
    }

    fn restore_file(&self, target: &Path) -> Result<()> {
        let backup = backup_path(target);
        if backup.exists() {
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
