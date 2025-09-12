/// replace this file with src/fs/restore/{mod.rs,types.rs,selector.rs,idempotence.rs,integrity.rs,steps.rs,engine.rs} — split monolith per zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md
//! Restore logic extracted from symlink.rs, using backup sidecar schema.

use std::fs;
use std::path::{Path, PathBuf};

use rustix::fs::{fchmod, openat, renameat, unlinkat, AtFlags, Mode, OFlags};

use super::atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};
use super::backup::{
    find_latest_backup_and_sidecar, find_previous_backup_and_sidecar, read_sidecar,
};
use crate::types::safepath::SafePath;

/// Restore a file from its backup. When no backup exists, return an error unless force_best_effort is true.
pub fn restore_file(
    target: &SafePath,
    dry_run: bool,
    force_best_effort: bool,
    backup_tag: &str,
) -> std::io::Result<()> {
    let target_path = target.as_path();
    // Locate latest backup payload and sidecar
    let pair = find_latest_backup_and_sidecar(&target_path, backup_tag);
    let (backup_opt, sidecar_path) = match pair {
        Some(p) => p,
        None => {
            // No artifacts at all
            if !force_best_effort {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "backup missing",
                ));
            } else {
                return Ok(());
            }
        }
    };

    // Read sidecar if present; if missing fall back to legacy rename behavior when backup payload exists
    let sc = read_sidecar(&sidecar_path).ok();

    // Idempotent short-circuit when sidecar exists and current state matches prior_kind
    if let Some(ref side) = sc {
        let kind_now = match std::fs::symlink_metadata(&target_path) {
            Ok(md) => {
                let ft = md.file_type();
                if ft.is_symlink() {
                    "symlink"
                } else if ft.is_file() {
                    "file"
                } else {
                    "other"
                }
            }
            Err(_) => "none",
        };
        match side.prior_kind.as_str() {
            "file" if kind_now == "file" => return Ok(()),
            "symlink" if kind_now == "symlink" => {
                if let Ok(cur) = std::fs::read_link(&target_path) {
                    let want = side.prior_dest.as_ref().map(|s| PathBuf::from(s));
                    if let Some(w) = want {
                        // Compare resolved forms for robustness
                        let mut cur_res = cur.clone();
                        if cur_res.is_relative() {
                            if let Some(parent) = target_path.parent() {
                                cur_res = parent.join(cur_res);
                            }
                        }
                        let mut want_res = w.clone();
                        if want_res.is_relative() {
                            if let Some(parent) = target_path.parent() {
                                want_res = parent.join(want_res);
                            }
                        }
                        let cur_res = std::fs::canonicalize(&cur_res).unwrap_or(cur_res);
                        let want_res = std::fs::canonicalize(&want_res).unwrap_or(want_res);
                        if cur_res == want_res {
                            return Ok(());
                        }
                    }
                }
            }
            "none" if kind_now == "none" => return Ok(()),
            _ => {}
        }
    }

    if dry_run {
        return Ok(());
    }

    // Perform restore per prior_kind when sidecar is present
    if let Some(side) = sc {
        match side.prior_kind.as_str() {
            "file" => {
                // Need backup payload to restore bytes
                let backup = match backup_opt {
                    Some(p) => p,
                    None => {
                        // Without payload, either best-effort noop or error
                        if force_best_effort {
                            return Ok(());
                        }
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "backup payload missing",
                        ));
                    }
                };
                // If sidecar includes a payload hash, verify integrity before restore.
                if let Some(ref expected) = side.payload_hash {
                    if let Some(actual) = crate::fs::meta::sha256_hex_of(&backup) {
                        if actual != *expected {
                            if force_best_effort {
                                // Best-effort: skip integrity-enforced restore
                                return Ok(());
                            }
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "backup payload hash mismatch",
                            ));
                        }
                    }
                }
                let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
                let fname = target_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("target");
                let bname = backup
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("backup");
                let dirfd = open_dir_nofollow(parent)?;
                let _ = fs::remove_file(&target_path);
                let old_c = std::ffi::CString::new(bname).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                })?;
                let new_c = std::ffi::CString::new(fname).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                })?;
                renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                    .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                // Restore mode when present
                if let Some(ms) = side.mode.as_ref() {
                    if let Ok(m) = u32::from_str_radix(ms, 8) {
                        let fname_c = std::ffi::CString::new(fname).map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                        })?;
                        let tfd = openat(&dirfd, fname_c.as_c_str(), OFlags::RDONLY, Mode::empty())
                            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                        let _ = fchmod(&tfd, Mode::from_bits_truncate(m));
                    }
                }
                let _ = fsync_parent_dir(&target_path);
            }
            "symlink" => {
                // Restore symlink to prior_dest
                if let Some(dest) = side.prior_dest.as_ref() {
                    let src = Path::new(dest);
                    let _ = atomic_symlink_swap(src, &target_path, true)?;
                    let _ = fsync_parent_dir(&target_path);
                    // Remove backup payload if present (sidecar remains)
                    if let Some(b) = backup_opt.as_ref() {
                        let _ = std::fs::remove_file(b);
                    }
                } else {
                    // Malformed sidecar; fallback to backup payload rename if available
                    if let Some(backup) = backup_opt {
                        let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
                        let fname = target_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("target");
                        let bname = backup
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("backup");
                        let dirfd = open_dir_nofollow(parent)?;
                        let _ = fs::remove_file(&target_path);
                        let old_c = std::ffi::CString::new(bname).map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                        })?;
                        let new_c = std::ffi::CString::new(fname).map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                        })?;
                        renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                        let _ = fsync_parent_dir(&target_path);
                    } else if !force_best_effort {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "backup payload missing",
                        ));
                    }
                }
            }
            "none" => {
                // Ensure path is absent
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
                } else {
                    let _ = std::fs::remove_file(&target_path);
                }
                let _ = fsync_parent_dir(&target_path);
                // Remove backup payload if present
                if let Some(b) = backup_opt.as_ref() {
                    let _ = std::fs::remove_file(b);
                }
            }
            _ => {
                // Unknown kind; fall back to legacy behavior when payload present
                if let Some(backup) = backup_opt {
                    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
                    let fname = target_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("target");
                    let bname = backup
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("backup");
                    let dirfd = open_dir_nofollow(parent)?;
                    let _ = fs::remove_file(&target_path);
                    let old_c = std::ffi::CString::new(bname).map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                    })?;
                    let new_c = std::ffi::CString::new(fname).map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                    })?;
                    renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                        .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                    let _ = fsync_parent_dir(&target_path);
                } else if !force_best_effort {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "backup payload missing",
                    ));
                }
            }
        }
        return Ok(());
    }

    // No sidecar: legacy rename if backup payload exists
    if let Some(backup) = backup_opt {
        if dry_run {
            return Ok(());
        }
        let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
        let fname = target_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("target");
        let bname = backup
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("backup");
        let _ = fs::remove_file(&target_path);
        let dirfd = open_dir_nofollow(parent)?;
        let old_c = std::ffi::CString::new(bname).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
        })?;
        let new_c = std::ffi::CString::new(fname).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
        })?;
        renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
        let _ = fsync_parent_dir(&target_path);
        Ok(())
    } else if force_best_effort {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "backup missing",
        ))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_BACKUP_TAG;
    use crate::fs::backup::find_latest_backup_and_sidecar;
    use crate::fs::swap::replace_file_with_symlink;
    use crate::types::safepath::SafePath;

    fn td() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn symlink_topology_restore_roundtrip() {
        let t = td();
        let root = t.path();
        let src_old = root.join("old");
        let src_new = root.join("new");
        let tgt = root.join("bin");
        std::fs::write(&src_old, b"old").unwrap();
        std::fs::write(&src_new, b"new").unwrap();
        // Start with target being a symlink to old (relative link)
        let _ = std::os::unix::fs::symlink("old", &tgt);

        // SafePaths
        let sp_src_new = SafePath::from_rooted(root, &src_new).unwrap();
        let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();

        // Replace target to point to new; backup should capture prior symlink target
        let _ = replace_file_with_symlink(&sp_src_new, &sp_tgt, false, false, DEFAULT_BACKUP_TAG)
            .unwrap();
        // Now restore
        restore_file(&sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        let md2 = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(
            md2.file_type().is_symlink(),
            "target should be symlink after restore"
        );
        let link = std::fs::read_link(&tgt).unwrap();
        assert_eq!(link, PathBuf::from("old"));
    }

    #[test]
    fn none_topology_restore_removes_target() {
        let t = td();
        let root = t.path();
        let src_new = root.join("new");
        let tgt = root.join("bin");
        std::fs::write(&src_new, b"new").unwrap();
        // Target does not exist initially
        assert!(!tgt.exists());
        // SafePaths
        let sp_src_new = SafePath::from_rooted(root, &src_new).unwrap();
        let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
        let _ = replace_file_with_symlink(&sp_src_new, &sp_tgt, false, false, DEFAULT_BACKUP_TAG)
            .unwrap();
        assert!(tgt.exists());
        // Restore should remove target (prior_kind=none)
        restore_file(&sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        assert!(
            !tgt.exists(),
            "target should be absent after restore of prior_kind=none"
        );
    }

    #[test]
    fn idempotent_restore_file_twice_is_noop() {
        let t = td();
        let root = t.path();
        let src = root.join("src");
        let tgt = root.join("tgt");
        std::fs::write(&src, b"new").unwrap();
        std::fs::write(&tgt, b"old").unwrap();
        let sp_src = SafePath::from_rooted(root, &src).unwrap();
        let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
        let _ =
            replace_file_with_symlink(&sp_src, &sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        restore_file(&sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        // Second restore should be a no-op
        restore_file(&sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        let md = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(md.file_type().is_file());
        let content = std::fs::read_to_string(&tgt).unwrap();
        assert!(content.starts_with("old"));
    }

    #[test]
    fn integrity_mismatch_fails_restore() {
        let t = td();
        let root = t.path();
        let src = root.join("src.bin");
        let tgt = root.join("tgt.bin");
        std::fs::write(&src, b"new").unwrap();
        std::fs::write(&tgt, b"old").unwrap();
        let sp_src = SafePath::from_rooted(root, &src).unwrap();
        let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
        // Perform swap to create snapshot and sidecar
        let _ = replace_file_with_symlink(&sp_src, &sp_tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        // Corrupt backup payload
        if let Some((Some(backup), _sc)) = find_latest_backup_and_sidecar(&tgt, DEFAULT_BACKUP_TAG) {
            // Overwrite payload bytes to mismatch hash
            let _ = std::fs::write(&backup, b"corrupt");
        }
        // Restore should now fail with NotFound (mapped to E_BACKUP_MISSING)
        let e = super::restore_file(&sp_tgt, false, false, DEFAULT_BACKUP_TAG).err().expect("should fail");
        assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        // Best-effort restore should succeed (no-op)
        assert!(super::restore_file(&sp_tgt, false, true, DEFAULT_BACKUP_TAG).is_ok());
    }
}

/// Restore from the previous (second newest) backup pair. Used when a fresh snapshot
/// was just captured pre-restore and we want to restore to the state before snapshot.
pub fn restore_file_prev(
    target: &SafePath,
    dry_run: bool,
    force_best_effort: bool,
    backup_tag: &str,
) -> std::io::Result<()> {
    let target_path = target.as_path();
    // Locate previous backup payload and sidecar
    let pair = find_previous_backup_and_sidecar(&target_path, backup_tag);
    let (backup_opt, sidecar_path) = match pair {
        Some(p) => p,
        None => {
            // No previous artifacts
            if !force_best_effort {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "backup missing",
                ));
            } else {
                return Ok(());
            }
        }
    };

    // Read sidecar if present; if missing fall back to legacy rename behavior when backup payload exists
    let sc = read_sidecar(&sidecar_path).ok();

    // Idempotent short-circuit when sidecar exists and current state matches prior_kind
    if let Some(ref side) = sc {
        let kind_now = match std::fs::symlink_metadata(&target_path) {
            Ok(md) => {
                let ft = md.file_type();
                if ft.is_symlink() {
                    "symlink"
                } else if ft.is_file() {
                    "file"
                } else {
                    "other"
                }
            }
            Err(_) => "none",
        };
        match side.prior_kind.as_str() {
            "file" if kind_now == "file" => return Ok(()),
            "symlink" if kind_now == "symlink" => {
                if let Ok(cur) = std::fs::read_link(&target_path) {
                    let want = side.prior_dest.as_ref().map(|s| PathBuf::from(s));
                    if let Some(w) = want {
                        // Compare resolved forms for robustness
                        let mut cur_res = cur.clone();
                        if cur_res.is_relative() {
                            if let Some(parent) = target_path.parent() {
                                cur_res = parent.join(cur_res);
                            }
                        }
                        let mut want_res = w.clone();
                        if want_res.is_relative() {
                            if let Some(parent) = target_path.parent() {
                                want_res = parent.join(want_res);
                            }
                        }
                        let cur_res = std::fs::canonicalize(&cur_res).unwrap_or(cur_res);
                        let want_res = std::fs::canonicalize(&want_res).unwrap_or(want_res);
                        if cur_res == want_res {
                            return Ok(());
                        }
                    }
                }
            }
            "none" if kind_now == "none" => return Ok(()),
            _ => {}
        }
    }

    if dry_run {
        return Ok(());
    }

    // Perform restore per prior_kind when sidecar is present
    if let Some(side) = sc {
        match side.prior_kind.as_str() {
            "file" => {
                // Need backup payload to restore bytes
                let backup = match backup_opt {
                    Some(p) => p,
                    None => {
                        if force_best_effort {
                            return Ok(());
                        }
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "backup payload missing",
                        ));
                    }
                };
                // If sidecar includes a payload hash, verify integrity before restore.
                if let Some(ref expected) = side.payload_hash {
                    if let Some(actual) = crate::fs::meta::sha256_hex_of(&backup) {
                        if actual != *expected {
                            if force_best_effort {
                                return Ok(());
                            }
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "backup payload hash mismatch",
                            ));
                        }
                    }
                }
                let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
                let fname = target_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("target");
                let bname = backup
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("backup");
                let dirfd = open_dir_nofollow(parent)?;
                let _ = fs::remove_file(&target_path);
                let old_c = std::ffi::CString::new(bname).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                })?;
                let new_c = std::ffi::CString::new(fname).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                })?;
                renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                    .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                // Restore mode when present
                if let Some(ms) = side.mode.as_ref() {
                    if let Ok(m) = u32::from_str_radix(ms, 8) {
                        let fname_c = std::ffi::CString::new(fname).map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                        })?;
                        let tfd = openat(&dirfd, fname_c.as_c_str(), OFlags::RDONLY, Mode::empty())
                            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                        let _ = fchmod(&tfd, Mode::from_bits_truncate(m));
                    }
                }
                let _ = fsync_parent_dir(&target_path);
            }
            "symlink" => {
                // Restore symlink to prior_dest
                if let Some(dest) = side.prior_dest.as_ref() {
                    let src = Path::new(dest);
                    let _ = atomic_symlink_swap(src, &target_path, true)?;
                    let _ = fsync_parent_dir(&target_path);
                    // Remove backup payload if present (sidecar remains)
                    if let Some(b) = backup_opt.as_ref() {
                        let _ = std::fs::remove_file(b);
                    }
                } else {
                    // Malformed sidecar; fallback to backup payload rename if available
                    if let Some(backup) = backup_opt {
                        let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
                        let fname = target_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("target");
                        let bname = backup
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("backup");
                        let dirfd = open_dir_nofollow(parent)?;
                        let _ = fs::remove_file(&target_path);
                        let old_c = std::ffi::CString::new(bname).map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                        })?;
                        let new_c = std::ffi::CString::new(fname).map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                        })?;
                        renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                        let _ = fsync_parent_dir(&target_path);
                    } else if !force_best_effort {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "backup payload missing",
                        ));
                    }
                }
            }
            "none" => {
                // Ensure path is absent
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
                } else {
                    let _ = std::fs::remove_file(&target_path);
                }
                let _ = fsync_parent_dir(&target_path);
                // Remove backup payload if present
                if let Some(b) = backup_opt.as_ref() {
                    let _ = std::fs::remove_file(b);
                }
            }
            _ => {
                // Unknown kind; fall back to legacy behavior when payload present
                if let Some(backup) = backup_opt {
                    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
                    let fname = target_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("target");
                    let bname = backup
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("backup");
                    let dirfd = open_dir_nofollow(parent)?;
                    let _ = fs::remove_file(&target_path);
                    let old_c = std::ffi::CString::new(bname).map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                    })?;
                    let new_c = std::ffi::CString::new(fname).map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
                    })?;
                    renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                        .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                    let _ = fsync_parent_dir(&target_path);
                } else if !force_best_effort {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "backup payload missing",
                    ));
                }
            }
        }
        return Ok(());
    }

    // No sidecar: legacy rename if backup payload exists
    if let Some(backup) = backup_opt {
        if dry_run {
            return Ok(());
        }
        let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
        let fname = target_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("target");
        let bname = backup
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("backup");
        let _ = fs::remove_file(&target_path);
        let dirfd = open_dir_nofollow(parent)?;
        let old_c = std::ffi::CString::new(bname).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
        })?;
        let new_c = std::ffi::CString::new(fname).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
        })?;
        renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
        let _ = fsync_parent_dir(&target_path);
        Ok(())
    } else if force_best_effort {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "backup missing",
        ))
    }
}
