//! Symlink swap orchestration that coordinates backup/snapshot and atomic swap.

use std::fs;

use rustix::fs::{unlinkat, AtFlags};

use super::atomic::{atomic_symlink_swap, open_dir_nofollow};
use super::backup::create_snapshot;
use crate::types::safepath::SafePath;

/// Atomically replace a file with a symlink, creating a backup. Emits no logs; pure mechanism.
/// Returns Ok(true) when degraded EXDEV fallback was used (non-atomic), Ok(false) otherwise.
pub fn replace_file_with_symlink(
    source: &SafePath,
    target: &SafePath,
    dry_run: bool,
    allow_degraded: bool,
    backup_tag: &str,
) -> std::io::Result<(bool, u64)> {
    let source_path = source.as_path();
    let target_path = target.as_path();

    if source_path == target_path {
        return Ok((false, 0));
    }

    if let Some(parent) = target_path.parent() {
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

    if dry_run {
        return Ok((false, 0));
    }

    if is_symlink {
        let desired = fs::canonicalize(&source_path).unwrap_or_else(|_| source_path.to_path_buf());
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
            let fname = target_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("target");
            let fname_c = std::ffi::CString::new(fname).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
            })?;
            let _ = unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty());
        }
        let res = atomic_symlink_swap(&source_path, &target_path, allow_degraded)?;
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
                let fname = target_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("target");
                let fname_c = std::ffi::CString::new(fname).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                })?;
                unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty())
                    .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
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
        let fname = target_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("target");
        let fname_c = std::ffi::CString::new(fname).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
        })?;
        let _ = unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty());
    }
    let res = atomic_symlink_swap(&source_path, &target_path, allow_degraded)?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_BACKUP_TAG;
    use crate::fs::restore::restore_file;
    use crate::types::safepath::SafePath;

    fn tmpdir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn atomic_swap_creates_symlink_pointing_to_source() {
        let td = tmpdir();
        let root = td.path();
        let src = root.join("source.txt");
        let tgt = root.join("target.txt");

        // Create source file
        std::fs::write(&src, b"hello").unwrap();

        // SafePaths
        let sp_src = SafePath::from_rooted(root, &src).unwrap();
        let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();

        // Perform atomic swap: create symlink at target -> source
        let _ =
            replace_file_with_symlink(&sp_src, &sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();

        // Verify target is a symlink pointing to source
        let md = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(md.file_type().is_symlink(), "target should be a symlink");
        let link = std::fs::read_link(&tgt).unwrap();
        assert_eq!(link, src);
    }

    #[test]
    fn replace_and_restore_roundtrip() {
        let td = tmpdir();
        let root = td.path();
        let src = root.join("bin-new");
        let tgt = root.join("bin-old");

        // Create source and target files
        std::fs::write(&src, b"new").unwrap();
        {
            let mut f = std::fs::File::create(&tgt).unwrap();
            use std::io::Write as _;
            writeln!(f, "old").unwrap();
        }

        let sp_src = SafePath::from_rooted(root, &src).unwrap();
        let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();

        // Replace target with symlink to source; backup should be created
        let _ =
            replace_file_with_symlink(&sp_src, &sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        let md = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(
            md.file_type().is_symlink(),
            "target should be a symlink after replace"
        );

        // Restore from backup; target should be a regular file again with prior content prefix
        restore_file(&sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        let md2 = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(
            md2.file_type().is_file(),
            "target should be a regular file after restore"
        );
        let content = std::fs::read_to_string(&tgt).unwrap();
        assert!(content.starts_with("old"));
    }
}
