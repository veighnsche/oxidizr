/// placeholder

use std::fs;
use std::time::Instant;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use libc;

use super::atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};

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
    for component in path.components() {
        if let std::path::Component::ParentDir = component { return false; }
    }
    if let Some(path_str) = path.to_str() {
        if path_str.contains("/../") || path_str.contains("..\\") { return false; }
    }
    true
}

/// Atomically replace a file with a symlink, creating a backup. Emits no logs; pure mechanism.
pub fn replace_file_with_symlink(source: &Path, target: &Path, dry_run: bool) -> std::io::Result<()> {
    if source == target { return Ok(()); }
    if !is_safe_path(source) || !is_safe_path(target) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "unsafe path"));
    }

    if let Some(parent) = target.parent() {
        let dirfd = open_dir_nofollow(parent)?;
        unsafe { libc::close(dirfd) };
    }

    let metadata = fs::symlink_metadata(target);
    let existed = metadata.is_ok();
    let is_symlink = metadata.as_ref().map(|m| m.file_type().is_symlink()).unwrap_or(false);
    let current_dest = if is_symlink { fs::read_link(target).ok() } else { None };

    if dry_run { return Ok(()); }

    if is_symlink {
        let desired = fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
        let mut resolved_current = current_dest.clone().unwrap_or_default();
        if resolved_current.is_relative() {
            if let Some(parent) = target.parent() { resolved_current = parent.join(resolved_current); }
        }
        let resolved_current = fs::canonicalize(&resolved_current).unwrap_or(resolved_current);
        if resolved_current == desired { return Ok(()); }

        // Backup symlink by creating a symlink backup pointing to the same destination
        if let Some(curr) = current_dest.as_ref() {
            let backup = backup_path(target);
            let _ = fs::remove_file(&backup);
            let t0 = Instant::now();
            let _ = unix_fs::symlink(curr, &backup);
            let _elapsed_ms = t0.elapsed().as_millis() as u64;
            let _ = _elapsed_ms; // reserved for future telemetry
        }
        // Atomically swap
        let _ = fs::remove_file(target);
        atomic_symlink_swap(source, target)?;
        return Ok(());
    }

    // Regular file: backup then replace with symlink
    if existed {
        let backup = backup_path(target);
        if let Ok(ref meta) = metadata {
            let t0 = Instant::now();
            fs::copy(target, &backup)?;
            let perm = meta.permissions();
            fs::set_permissions(&backup, perm)?;
            fs::remove_file(target)?;
            let _elapsed_ms = t0.elapsed().as_millis() as u64;
            let _ = _elapsed_ms; // reserved for future telemetry
        }
    }

    if let Some(parent) = target.parent() { fs::create_dir_all(parent)?; }
    let _ = fs::remove_file(target);
    atomic_symlink_swap(source, target)?;
    Ok(())
}

/// Restore a file from its backup. When no backup exists, return an error unless force_best_effort is true.
pub fn restore_file(target: &Path, dry_run: bool, force_best_effort: bool) -> std::io::Result<()> {
    let backup = backup_path(target);
    if backup.exists() {
        if dry_run { return Ok(()); }
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
        let bname = backup.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
        let _ = fs::remove_file(target);
        let dirfd = open_dir_nofollow(parent)?;
        let old_c = std::ffi::CString::new(bname).unwrap();
        let new_c = std::ffi::CString::new(fname).unwrap();
        let rc = unsafe { libc::renameat(dirfd, old_c.as_ptr(), dirfd, new_c.as_ptr()) };
        let last = std::io::Error::last_os_error();
        unsafe { libc::close(dirfd) };
        if rc != 0 { return Err(last); }
        let _ = fsync_parent_dir(target);
    } else {
        if !force_best_effort {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup missing"));
        }
    }
    Ok(())
}
