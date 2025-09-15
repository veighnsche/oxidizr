//! Symlink swap orchestration that coordinates backup/snapshot and atomic swap.

use std::fs;
use std::os::unix::ffi::OsStrExt;

use rustix::fs::{unlinkat, AtFlags};

use super::atomic::{atomic_symlink_swap, open_dir_nofollow};
use super::backup::create_snapshot;
use crate::types::safepath::SafePath;

/// Atomically replace a file with a symlink, creating a backup. Emits no logs; pure mechanism.
/// Returns Ok(true) when degraded EXDEV fallback was used (non-atomic), Ok(false) otherwise.
///
/// # Errors
///
/// Returns an IO error if the file cannot be replaced with a symlink or if backup creation fails.
pub fn replace_file_with_symlink(
    source: &SafePath,
    target: &SafePath,
    dry_run: bool,
    allow_degraded: bool,
    backup_tag: &str,
) -> std::io::Result<(bool, u64)> {
    replace_file_with_symlink_with_override(
        source,
        target,
        dry_run,
        allow_degraded,
        backup_tag,
        None,
    )
}

/// Version of `replace_file_with_symlink` that accepts a per-instance EXDEV override for tests/controlled scenarios.
///
/// # Errors
///
/// Returns an IO error when:
/// - Capability handle acquisition for the parent directory fails
/// - Removing the existing target fails with an unexpected errno
/// - Creating a snapshot (backup) of the prior state fails (Commit mode)
/// - Performing the atomic symlink swap fails (including EXDEV fallback when disallowed)
#[allow(
    clippy::too_many_lines,
    reason = "Will be broken down into smaller helpers in a follow-up refactor"
)]
pub fn replace_file_with_symlink_with_override(
    source: &SafePath,
    target: &SafePath,
    dry_run: bool,
    allow_degraded: bool,
    backup_tag: &str,
    force_exdev: Option<bool>,
) -> std::io::Result<(bool, u64)> {
    let source_path = source.as_path();
    let target_path = target.as_path();

    // In DryRun, avoid any filesystem I/O and return immediately. This ensures
    // that redacted, deterministic facts can be emitted without requiring the
    // target directories to exist or be accessible.
    if dry_run {
        return Ok((false, 0));
    }

    if source_path == target_path {
        return Ok((false, 0));
    }

    // Ensure parent directory exists prior to acquiring a no‑follow dir handle.
    // This avoids failures when committing a plan in a fresh temp root where
    // target parents (e.g., usr/bin) may not yet exist but are implied by the plan.
    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
        let _dirfd = open_dir_nofollow(parent)?; // RAII drop closes
    }

    let metadata = fs::symlink_metadata(&target_path);
    let existed = metadata.is_ok();
    let is_symlink = metadata
        .as_ref()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);
    let current_dest = if is_symlink {
        fs::read_link(&target_path).ok()
    } else {
        None
    };

    // DryRun already handled above; proceed with real operations.

    if is_symlink {
        let desired = fs::canonicalize(&source_path).unwrap_or_else(|_| source_path.clone());
        let mut resolved_current = current_dest.clone().unwrap_or_default();
        if resolved_current.is_relative() {
            if let Some(parent) = target_path.parent() {
                resolved_current = parent.join(resolved_current);
            }
        }
        let resolved_current = fs::canonicalize(&resolved_current).unwrap_or(resolved_current);
        if resolved_current == desired {
            return Ok((false, 0));
        }

        // Snapshot current symlink topology before mutation
        if let Err(e) = create_snapshot(&target_path, backup_tag) {
            if !dry_run {
                return Err(e);
            }
        }
        // Atomically swap: ensure target removed via cap-handle
        if let Some(parent) = target_path.parent() {
            let dirfd = open_dir_nofollow(parent)?;
            // Bytes-safe C string from OsStr
            let fname_c = if let Some(name_os) = target_path.file_name() {
                std::ffi::CString::new(name_os.as_bytes()).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                })?
            } else {
                std::ffi::CString::new("target").map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                })?
            };
            match unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty()) {
                Ok(()) => {}
                Err(e) if e == rustix::io::Errno::NOENT => {}
                Err(e) => return Err(std::io::Error::from_raw_os_error(e.raw_os_error())),
            }
        }
        let res = atomic_symlink_swap(&source_path, &target_path, allow_degraded, force_exdev)?;
        return Ok(res);
    }

    // Regular file: backup then replace with symlink
    if existed {
        if let Ok(_meta) = metadata {
            // Snapshot current file state before mutation
            if let Err(e) = create_snapshot(&target_path, backup_tag) {
                if !dry_run {
                    return Err(e);
                }
            }
            // Remove original target (will be replaced by atomic swap below)
            if let Some(parent) = target_path.parent() {
                let dirfd = open_dir_nofollow(parent)?;
                let fname_c = if let Some(name_os) = target_path.file_name() {
                    std::ffi::CString::new(name_os.as_bytes()).map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                    })?
                } else {
                    std::ffi::CString::new("target").map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                    })?
                };
                match unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty()) {
                    Ok(()) => {}
                    Err(e) if e == rustix::io::Errno::NOENT => {}
                    Err(e) => return Err(std::io::Error::from_raw_os_error(e.raw_os_error())),
                }
            } else {
                let _ = fs::remove_file(&target_path);
            }
        }
    } else {
        // Create tombstone snapshot
        if let Err(e) = create_snapshot(&target_path, backup_tag) {
            if !dry_run {
                return Err(e);
            }
        }
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Ensure target removed via capability handle
    if let Some(parent) = target_path.parent() {
        let dirfd = open_dir_nofollow(parent)?;
        let fname_c = if let Some(name_os) = target_path.file_name() {
            std::ffi::CString::new(name_os.as_bytes()).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
            })?
        } else {
            std::ffi::CString::new("target").map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
            })?
        };
        match unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty()) {
            Ok(()) => {}
            Err(e) if e == rustix::io::Errno::NOENT => {}
            Err(e) => return Err(std::io::Error::from_raw_os_error(e.raw_os_error())),
        }
    }
    let res = atomic_symlink_swap(&source_path, &target_path, allow_degraded, force_exdev)?;
    Ok(res)
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_BACKUP_TAG;
    use crate::fs::restore::restore_file;
    use crate::types::safepath::SafePath;
    use std::io::Write;

    fn tmpdir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap_or_else(|_| panic!("Failed to create tempdir"))
    }

    #[test]
    fn atomic_swap_creates_symlink_pointing_to_source() {
        let td = tmpdir();
        let root = td.path();
        let src = root.join("source.txt");
        let tgt = root.join("target.txt");

        // Create source file
        fs::write(&src, b"hello").expect("Failed to write source file");

        // SafePaths
        let sp_src = SafePath::from_rooted(root, &src).expect("Failed to create source SafePath");
        let sp_tgt = SafePath::from_rooted(root, &tgt).expect("Failed to create target SafePath");

        // Perform atomic swap: create symlink at target -> source
        let _ = replace_file_with_symlink(&sp_src, &sp_tgt, false, false, DEFAULT_BACKUP_TAG)
            .expect("Failed to replace file with symlink");

        // Verify target is a symlink pointing to source
        let md = fs::symlink_metadata(&tgt).expect("Failed to get symlink metadata");
        assert!(md.file_type().is_symlink(), "target should be a symlink");
        let link = fs::read_link(&tgt).expect("Failed to read symlink");
        assert_eq!(link, src);
    }

    #[test]
    fn replace_and_restore_roundtrip() {
        let td = tmpdir();
        let root = td.path();
        let src = root.join("bin-new");
        let tgt = root.join("bin-old");

        // Create source and target files
        fs::write(&src, b"new").unwrap_or_else(|e| panic!("Failed to write source file: {e}"));
        {
            let mut f = fs::File::create(&tgt)
                .unwrap_or_else(|e| panic!("Failed to create target file: {e}"));
            writeln!(f, "old").unwrap_or_else(|e| panic!("Failed to write to target file: {e}"));
        }

        let sp_src = SafePath::from_rooted(root, &src)
            .unwrap_or_else(|e| panic!("Failed to create source SafePath: {e}"));
        let sp_tgt = SafePath::from_rooted(root, &tgt)
            .unwrap_or_else(|e| panic!("Failed to create target SafePath: {e}"));

        // Replace target with symlink to source; backup should be created
        let _ = replace_file_with_symlink(&sp_src, &sp_tgt, false, false, DEFAULT_BACKUP_TAG)
            .unwrap_or_else(|e| panic!("Failed to replace file with symlink: {e}"));
        let md = fs::symlink_metadata(&tgt)
            .unwrap_or_else(|e| panic!("Failed to get symlink metadata: {e}"));
        assert!(
            md.file_type().is_symlink(),
            "target should be a symlink after replace"
        );

        // Restore from backup; target should be a regular file again with prior content prefix
        restore_file(&sp_tgt, false, false, DEFAULT_BACKUP_TAG)
            .unwrap_or_else(|e| panic!("Failed to restore file: {e}"));
        let md2 = fs::symlink_metadata(&tgt)
            .unwrap_or_else(|e| panic!("Failed to get symlink metadata: {e}"));
        assert!(
            md2.file_type().is_file(),
            "target should be a regular file after restore"
        );
        let content =
            fs::read_to_string(&tgt).unwrap_or_else(|e| panic!("Failed to read target file: {e}"));
        assert!(content.starts_with("old"));
    }
}
