//! Restore logic extracted from symlink.rs, using backup sidecar schema.

use std::path::{Path, PathBuf};
use std::fs;

use rustix::fs::{fchmod, openat, renameat, unlinkat, AtFlags, Mode, OFlags};

use super::atomic::{fsync_parent_dir, open_dir_nofollow, atomic_symlink_swap};
use super::backup::{find_latest_backup_and_sidecar, read_sidecar};

/// Restore a file from its backup. When no backup exists, return an error unless force_best_effort is true.
pub fn restore_file(
    target: &Path,
    dry_run: bool,
    force_best_effort: bool,
    backup_tag: &str,
) -> std::io::Result<()> {
    // Locate latest backup payload and sidecar
    let pair = find_latest_backup_and_sidecar(target, backup_tag);
    let (backup_opt, sidecar_path) = match pair {
        Some(p) => p,
        None => {
            // No artifacts at all
            if !force_best_effort {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup missing"));
            } else {
                return Ok(());
            }
        }
    };

    // Read sidecar if present; if missing fall back to legacy rename behavior when backup payload exists
    let sc = read_sidecar(&sidecar_path).ok();

    // Idempotent short-circuit when sidecar exists and current state matches prior_kind
    if let Some(ref side) = sc {
        let kind_now = match std::fs::symlink_metadata(target) {
            Ok(md) => {
                let ft = md.file_type();
                if ft.is_symlink() { "symlink" } else if ft.is_file() { "file" } else { "other" }
            }
            Err(_) => "none",
        };
        match side.prior_kind.as_str() {
            "file" if kind_now == "file" => return Ok(()),
            "symlink" if kind_now == "symlink" => {
                if let Ok(cur) = std::fs::read_link(target) {
                    let want = side.prior_dest.as_ref().map(|s| PathBuf::from(s));
                    if let Some(w) = want {
                        // Compare resolved forms for robustness
                        let mut cur_res = cur.clone();
                        if cur_res.is_relative() {
                            if let Some(parent) = target.parent() { cur_res = parent.join(cur_res); }
                        }
                        let mut want_res = w.clone();
                        if want_res.is_relative() {
                            if let Some(parent) = target.parent() { want_res = parent.join(want_res); }
                        }
                        let cur_res = std::fs::canonicalize(&cur_res).unwrap_or(cur_res);
                        let want_res = std::fs::canonicalize(&want_res).unwrap_or(want_res);
                        if cur_res == want_res { return Ok(()); }
                    }
                }
            }
            "none" if kind_now == "none" => return Ok(()),
            _ => {}
        }
    }

    if dry_run { return Ok(()); }

    // Perform restore per prior_kind when sidecar is present
    if let Some(side) = sc {
        match side.prior_kind.as_str() {
            "file" => {
                // Need backup payload to restore bytes
                let backup = match backup_opt {
                    Some(p) => p,
                    None => {
                        // Without payload, either best-effort noop or error
                        if force_best_effort { return Ok(()); }
                        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup payload missing"));
                    }
                };
                let parent = target.parent().unwrap_or_else(|| Path::new("."));
                let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
                let bname = backup.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
                let dirfd = open_dir_nofollow(parent)?;
                let _ = fs::remove_file(target);
                let old_c = std::ffi::CString::new(bname)
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
                let new_c = std::ffi::CString::new(fname)
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
                renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                    .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                // Restore mode when present
                if let Some(ms) = side.mode.as_ref() {
                    if let Ok(m) = u32::from_str_radix(ms, 8) {
                        let fname_c = std::ffi::CString::new(fname)
                            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
                        let tfd = openat(&dirfd, fname_c.as_c_str(), OFlags::RDONLY, Mode::empty())
                            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                        let _ = fchmod(&tfd, Mode::from_bits_truncate(m));
                    }
                }
                let _ = fsync_parent_dir(target);
            }
            "symlink" => {
                // Restore symlink to prior_dest
                if let Some(dest) = side.prior_dest.as_ref() {
                    let src = Path::new(dest);
                    let _ = atomic_symlink_swap(src, target, true)?;
                    let _ = fsync_parent_dir(target);
                    // Remove backup payload if present (sidecar remains)
                    if let Some(b) = backup_opt.as_ref() { let _ = std::fs::remove_file(b); }
                } else {
                    // Malformed sidecar; fallback to backup payload rename if available
                    if let Some(backup) = backup_opt {
                        let parent = target.parent().unwrap_or_else(|| Path::new("."));
                        let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
                        let bname = backup.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
                        let dirfd = open_dir_nofollow(parent)?;
                        let _ = fs::remove_file(target);
                        let old_c = std::ffi::CString::new(bname)
                            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
                        let new_c = std::ffi::CString::new(fname)
                            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
                        renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                        let _ = fsync_parent_dir(target);
                    } else if !force_best_effort {
                        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup payload missing"));
                    }
                }
            }
            "none" => {
                // Ensure path is absent
                if let Some(parent) = target.parent() {
                    let dirfd = open_dir_nofollow(parent)?;
                    let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
                    let fname_c = std::ffi::CString::new(fname)
                        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
                    let _ = unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty());
                } else {
                    let _ = std::fs::remove_file(target);
                }
                let _ = fsync_parent_dir(target);
                // Remove backup payload if present
                if let Some(b) = backup_opt.as_ref() { let _ = std::fs::remove_file(b); }
            }
            _ => {
                // Unknown kind; fall back to legacy behavior when payload present
                if let Some(backup) = backup_opt {
                    let parent = target.parent().unwrap_or_else(|| Path::new("."));
                    let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
                    let bname = backup.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
                    let dirfd = open_dir_nofollow(parent)?;
                    let _ = fs::remove_file(target);
                    let old_c = std::ffi::CString::new(bname)
                        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
                    let new_c = std::ffi::CString::new(fname)
                        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
                    renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                        .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                    let _ = fsync_parent_dir(target);
                } else if !force_best_effort {
                    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup payload missing"));
                }
            }
        }
        return Ok(());
    }

    // No sidecar: legacy rename if backup payload exists
    if let Some(backup) = backup_opt {
        if dry_run { return Ok(()); }
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
        let bname = backup.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
        let _ = fs::remove_file(target);
        let dirfd = open_dir_nofollow(parent)?;
        let old_c = std::ffi::CString::new(bname)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
        let new_c = std::ffi::CString::new(fname)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
        renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
        let _ = fsync_parent_dir(target);
        Ok(())
    } else if force_best_effort {
        Ok(())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup missing"))
    }
}
