use crate::logging::{audit_op, audit_event_fields, AuditFields};
use crate::ui::progress::symlink_info_enabled;
use crate::Result;
use std::fs;
use std::time::Instant;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::os::unix::io::RawFd;
use nix::libc;

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
        if symlink_info_enabled() {
            tracing::info!(
                "Source and target are the same ({}), skipping symlink.",
                source.display()
            );
        }
        return Ok(());
    }

    // Validate paths to prevent directory traversal attacks
    if !is_safe_path(source) || !is_safe_path(target) {
        return Err(crate::Error::ExecutionFailed(
            "Invalid path: contains directory traversal".into(),
        ));
    }

    // Enforce no-follow on the parent directory using open(O_DIRECTORY|O_NOFOLLOW)
    if let Some(parent) = target.parent() {
        let _dirfd = open_dir_nofollow(parent).map_err(|e| {
            crate::Error::ExecutionFailed(format!(
                "failed parent no-follow open on {}: {}",
                parent.display(),
                e
            ))
        })?;
        // Close immediately; existence and no-follow guarantees are enough here.
        unsafe { libc::close(_dirfd) };
    }

    // Use symlink_metadata to avoid simple TOCTOU races on the leaf
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

    if symlink_info_enabled() {
        tracing::info!(
            "replace_file_with_symlink pre-state: target={}, existed={}, is_symlink={}, current_dest={}",
            target.display(),
            existed,
            is_symlink,
            current_dest
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<none>".into())
        );
    }

    if dry_run {
        if symlink_info_enabled() {
            tracing::info!(
                "[dry-run] would ensure symlink {} -> {} (updating/replacing as needed)",
                source.display(),
                target.display()
            );
        }
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
            if symlink_info_enabled() {
                tracing::info!(
                    "Existing symlink already correct: {} -> {} (no action)",
                    target.display(),
                    resolved_current.display()
                );
            }
            return Ok(());
        } else {
            if symlink_info_enabled() {
                tracing::info!(
                    "Replacing existing symlink: {} currently -> {}, desired -> {}",
                    target.display(),
                    current_dest
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<unreadable>".into()),
                    source.display()
                );
            }

            // Link-aware backup: back up the symlink itself (if present) by creating a backup symlink
            let backup = backup_path(target);
            if is_symlink {
                if let Some(curr) = current_dest.as_ref() {
                    let _ = fs::remove_file(&backup);
                    if symlink_info_enabled() {
                        tracing::info!(
                            "Backing up symlink {} (-> {}) as {}",
                            target.display(),
                            curr.display(),
                            backup.display()
                        );
                    }
                    let t0 = Instant::now();
                    // Create a symlink backup pointing to the same destination
                    let _ = unix_fs::symlink(curr, &backup);
                    let elapsed_ms = t0.elapsed().as_millis() as u64;
                    let _ = audit_event_fields(
                        "symlink",
                        "backup_created",
                        "success",
                        &AuditFields {
                            backup_path: Some(backup.display().to_string()),
                            source: Some(curr.display().to_string()),
                            target: Some(target.display().to_string()),
                            duration_ms: Some(elapsed_ms),
                            ..Default::default()
                        },
                    );
                }
            }
            // Atomically swap using a temp symlink + rename
            fs::remove_file(target)?;
            atomic_symlink_swap(source, target)?;
            if symlink_info_enabled() {
                tracing::info!(
                    "Symlink updated: {} -> {}",
                    target.display(),
                    source.display()
                );
            }
            return Ok(());
        }
    }

    // For regular files: backup then replace with symlink atomically
    if existed {
        let backup = backup_path(target);
        if symlink_info_enabled() {
            tracing::info!("Backing up {} -> {}", target.display(), backup.display());
        }
        // Use metadata we already have to avoid additional TOCTOU
        if let Ok(ref meta) = metadata {
            let t0 = Instant::now();
            fs::copy(target, &backup)?;
            let perm = meta.permissions();
            fs::set_permissions(&backup, perm)?;
            fs::remove_file(target)?;
            let elapsed_ms = t0.elapsed().as_millis() as u64;
            let _ = audit_event_fields(
                "symlink",
                "backup_created",
                "success",
                &AuditFields {
                    backup_path: Some(backup.display().to_string()),
                    target: Some(target.display().to_string()),
                    duration_ms: Some(elapsed_ms),
                    ..Default::default()
                },
            );
        } else {
            // If metadata failed, the file might have been removed - handle gracefully
            tracing::warn!(
                "Target file {} disappeared during operation",
                target.display()
            );
        }
    }

    // Ensure parent exists
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    // Remove leftover target then perform atomic symlink swap
    let _ = fs::remove_file(target);
    atomic_symlink_swap(source, target)?;
    if symlink_info_enabled() {
        tracing::info!(
            "Symlink created: {} -> {}",
            target.display(),
            source.display()
        );
    }

    // Log the symlink creation
    let _ = audit_op(
        "CREATE_SYMLINK",
        &format!("{} -> {}", target.display(), source.display()),
        true,
    );

    Ok(())
}

/// Restore a file from its backup. When no backup exists, return an error unless force_best_effort is true.
pub fn restore_file(target: &Path, dry_run: bool, force_best_effort: bool) -> Result<()> {
    let backup = backup_path(target);
    if backup.exists() {
        if dry_run {
            if symlink_info_enabled() {
                tracing::info!(
                    "[dry-run] would restore {} from {}",
                    target.display(),
                    backup.display()
                );
            }
            return Ok(());
        }
        if symlink_info_enabled() {
            tracing::info!("Restoring {} <- {}", target.display(), backup.display());
        }
        // Remove current target then atomically rename backup into place
        let _ = fs::remove_file(target);
        fs::rename(&backup, target)?;
        // fsync parent directory to solidify rename
        let _ = fsync_parent_dir(target);
        // Log the restoration
        let _ = audit_op("RESTORE_FILE", &format!("{}", target.display()), true);
    } else {
        if force_best_effort {
            tracing::warn!("No backup for {}, leaving as-is", target.display());
        } else {
            return Err(crate::Error::RestoreBackupMissing(
                target.display().to_string(),
            ));
        }
    }
    Ok(())
}

/// Create a symlink at a temporary path in the same directory as target and atomically rename it into place.
fn atomic_symlink_swap(source: &Path, target: &Path) -> std::io::Result<()> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let fname = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("target");
    let tmp_name = format!(".{}.oxidizr.tmp", fname);
    let tmp = parent.join(&tmp_name);
    // Best-effort cleanup
    let _ = fs::remove_file(&tmp);
    unix_fs::symlink(source, &tmp)?;

    // Perform renameat anchored at the parent directory to avoid path races
    let dirfd = open_dir_nofollow(parent)?;
    let old_c = std::ffi::CString::new(tmp_name.as_str()).unwrap();
    let new_c = std::ffi::CString::new(fname).unwrap();
    let rc = unsafe { libc::renameat(dirfd, old_c.as_ptr(), dirfd, new_c.as_ptr()) };
    let last = std::io::Error::last_os_error();
    unsafe { libc::close(dirfd) };
    if rc != 0 { return Err(last); }

    // fsync parent directory
    let _ = fsync_parent_dir(target);
    Ok(())
}

fn open_dir_nofollow(dir: &Path) -> std::io::Result<RawFd> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(dir.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid path"))?;
    // O_NOFOLLOW will fail with ELOOP if dir is a symlink
    let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW;
    let fd = unsafe { libc::open(c.as_ptr(), flags, 0) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(fd)
}

fn fsync_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        // Open the directory and fsync it
        let dir = std::fs::File::open(parent)?;
        dir.sync_all()?;
    }
    Ok(())
}
