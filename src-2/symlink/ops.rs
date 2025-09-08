use crate::Result;
use crate::logging::PROVENANCE;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

const BACKUP_SUFFIX: &str = ".oxidizr.bak";

/// Generate backup path for a target file
pub fn backup_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("backup");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{}{}", name, BACKUP_SUFFIX))
}

/// Validate path to prevent directory traversal attacks
pub fn is_safe_path(path: &Path) -> bool {
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

/// Atomically replace a file with a symlink, creating a backup
pub fn replace_file_with_symlink(source: &Path, target: &Path, dry_run: bool) -> Result<()> {
    if source == target {
        log::info!("Source and target are the same ({}), skipping symlink.", source.display());
        return Ok(());
    }

    // Validate paths to prevent directory traversal attacks
    if !is_safe_path(source) || !is_safe_path(target) {
        return Err(crate::Error::ExecutionFailed(
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

    if dry_run {
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
            if let Some(parent) = target.parent() {
                resolved_current = parent.join(resolved_current);
            }
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
            
            // Create a backup of the current resolved target if it exists
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
    
    // Log the symlink creation
    let _ = PROVENANCE.log_operation(
        "CREATE_SYMLINK",
        &format!("{} -> {}", target.display(), source.display()),
        true,
    );
    
    Ok(())
}

/// Restore a file from its backup
pub fn restore_file(target: &Path, dry_run: bool) -> Result<()> {
    let backup = backup_path(target);
    if backup.exists() {
        if dry_run {
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
        // Log the restoration
        let _ = PROVENANCE.log_operation("RESTORE_FILE", &format!("{}", target.display()), true);
    } else {
        log::warn!("No backup for {}, leaving as-is", target.display());
    }
    Ok(())
}
