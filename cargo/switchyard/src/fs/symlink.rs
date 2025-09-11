/// placeholder
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};
use rustix::fs::{fchmod, openat, renameat, unlinkat, AtFlags, Mode, OFlags};
use std::os::unix::fs::PermissionsExt;
use serde::{Deserialize, Serialize};

const DEFAULT_BACKUP_TAG: &str = "switchyard";

/// Generate a unique backup path for a target file (includes a timestamp).
pub fn backup_path_with_tag(target: &Path, tag: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("backup");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    parent.join(format!(".{}.{}.{}.bak", name, tag, ts))
}

/// Public helper for preflight/tests: check if there are backup artifacts (payload and/or sidecar)
/// for the given target and tag.
pub fn has_backup_artifacts(target: &Path, tag: &str) -> bool {
    if let Some((payload, sc)) = find_latest_backup_and_sidecar(target, tag) {
        payload.is_some() || sc.exists()
    } else { false }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupSidecar {
    schema: String,                   // "backup_meta.v1"
    prior_kind: String,               // "file" | "symlink" | "none"
    #[serde(skip_serializing_if = "Option::is_none")]
    prior_dest: Option<String>,       // for symlink
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,             // octal string for file, e.g. "100644"
}

fn sidecar_path_for_backup(backup: &Path) -> PathBuf {
    let mut s = backup.as_os_str().to_owned();
    use std::ffi::OsString;
    let mut s2 = OsString::from(s);
    s2.push(".meta.json");
    PathBuf::from(s2)
}

fn write_sidecar(backup: &Path, sc: &BackupSidecar) -> std::io::Result<()> {
    let sc_path = sidecar_path_for_backup(backup);
    if let Some(parent) = sc_path.parent() { std::fs::create_dir_all(parent)?; }
    let f = std::fs::File::create(&sc_path)?;
    serde_json::to_writer_pretty(f, sc).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

fn read_sidecar(sc_path: &Path) -> std::io::Result<BackupSidecar> {
    let f = std::fs::File::open(sc_path)?;
    serde_json::from_reader(f).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

// Return (backup_path_if_present, sidecar_path) for the latest timestamped pair.
pub(crate) fn find_latest_backup_and_sidecar(target: &Path, tag: &str) -> Option<(Option<PathBuf>, PathBuf)> {
    let name = target.file_name()?.to_str()?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{}.{}.", name, tag);
    let mut best: Option<(u128, PathBuf)> = None; // timestamp, base path (without .meta.json)
    if let Ok(rd) = std::fs::read_dir(parent) {
        for e in rd.flatten() {
            let fname = e.file_name();
            if let Some(s) = fname.to_str() {
                // Accept either .bak or .bak.meta.json
                if let Some(rest) = s.strip_prefix(&prefix) {
                    if let Some(num_s) = rest.strip_suffix(".bak").or_else(|| rest.strip_suffix(".bak.meta.json")) {
                        if let Ok(num) = num_s.parse::<u128>() {
                            if best.as_ref().map(|(b, _)| num > *b).unwrap_or(true) {
                                // compute base path ending with .bak
                                let base = parent.join(format!("{}.bak", prefix.clone() + &num.to_string()));
                                best = Some((num, base));
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some((_, base)) = best {
        let sc = sidecar_path_for_backup(&base);
        let backup_present = if base.exists() { Some(base) } else { None };
        Some((backup_present, sc))
    } else {
        None
    }
}

fn find_latest_backup(target: &Path, tag: &str) -> Option<PathBuf> {
    let name = target.file_name()?.to_str()?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{}.{}.", name, tag);
    let mut best: Option<(u128, PathBuf)> = None;
    if let Ok(rd) = std::fs::read_dir(parent) {
        for e in rd.flatten() {
            if let Ok(ft) = e.file_type() {
                if !ft.is_file() {
                    continue;
                }
            }
            let fname = e.file_name();
            if let Some(s) = fname.to_str() {
                if s.starts_with(&prefix) && s.ends_with(".bak") {
                    // parse the millis suffix between prefix and .bak
                    if let Some(num) = s
                        .strip_prefix(&prefix)
                        .and_then(|rest| rest.strip_suffix(".bak"))
                        .and_then(|n| n.parse::<u128>().ok())
                    {
                        if best.as_ref().map(|(b, _)| num > *b).unwrap_or(true) {
                            best = Some((num, e.path()));
                        }
                    }
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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

        // Perform atomic swap: create symlink at target -> source
        let _ = crate::fs::atomic::atomic_symlink_swap(&src, &tgt, false).unwrap();

        // Verify target is a symlink pointing to source
        let md = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(md.file_type().is_symlink(), "target should be a symlink");
        let link = std::fs::read_link(&tgt).unwrap();
        // Depending on platform, the link may be absolute (we pass absolute src), so compare directly
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
            writeln!(f, "old").unwrap();
        }

        // Replace target with symlink to source; backup should be created
        let _ = replace_file_with_symlink(&src, &tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        let md = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(
            md.file_type().is_symlink(),
            "target should be a symlink after replace"
        );
        let latest = find_latest_backup(&tgt, DEFAULT_BACKUP_TAG).expect("latest backup");
        assert!(latest.exists(), "latest backup should exist after replace");

        // Restore from backup; target should be a regular file again with prior content prefix
        restore_file(&tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        let md2 = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(
            md2.file_type().is_file(),
            "target should be a regular file after restore"
        );
        let content = std::fs::read_to_string(&tgt).unwrap();
        assert!(
            content.starts_with("old"),
            "restored content should match prior file"
        );
    }

    #[test]
    fn symlink_topology_restore_roundtrip() {
        let td = tmpdir();
        let root = td.path();
        let src_old = root.join("old");
        let src_new = root.join("new");
        let tgt = root.join("bin");
        std::fs::write(&src_old, b"old").unwrap();
        std::fs::write(&src_new, b"new").unwrap();
        // Start with target being a symlink to old (relative link)
        let _ = unix_fs::symlink("old", &tgt);

        // Replace target to point to new; backup should capture prior symlink target
        let _ = replace_file_with_symlink(&src_new, &tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        // Now restore
        restore_file(&tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        let md2 = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(md2.file_type().is_symlink(), "target should be symlink after restore");
        let link = std::fs::read_link(&tgt).unwrap();
        assert_eq!(link, PathBuf::from("old"));
    }

    #[test]
    fn none_topology_restore_removes_target() {
        let td = tmpdir();
        let root = td.path();
        let src_new = root.join("new");
        let tgt = root.join("bin");
        std::fs::write(&src_new, b"new").unwrap();
        // Target does not exist initially
        assert!(!tgt.exists());
        let _ = replace_file_with_symlink(&src_new, &tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        assert!(tgt.exists());
        // Restore should remove target (prior_kind=none)
        restore_file(&tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        assert!(!tgt.exists(), "target should be absent after restore of prior_kind=none");
    }

    #[test]
    fn idempotent_restore_file_twice_is_noop() {
        let td = tmpdir();
        let root = td.path();
        let src = root.join("src");
        let tgt = root.join("tgt");
        std::fs::write(&src, b"new").unwrap();
        std::fs::write(&tgt, b"old").unwrap();
        let _ = replace_file_with_symlink(&src, &tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        restore_file(&tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        // Second restore should be a no-op
        restore_file(&tgt, false, false, DEFAULT_BACKUP_TAG).unwrap();
        let md = std::fs::symlink_metadata(&tgt).unwrap();
        assert!(md.file_type().is_file());
        let content = std::fs::read_to_string(&tgt).unwrap();
        assert!(content.starts_with("old"));
    }
}

/// Validate path to prevent directory traversal attacks
pub fn is_safe_path(path: &Path) -> bool {
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return false;
        }
    }
    if let Some(path_str) = path.to_str() {
        if path_str.contains("/../") || path_str.contains("..\\") {
            return false;
        }
    }
    true
}

/// Atomically replace a file with a symlink, creating a backup. Emits no logs; pure mechanism.
/// Returns Ok(true) when degraded EXDEV fallback was used (non-atomic), Ok(false) otherwise.
pub fn replace_file_with_symlink(
    source: &Path,
    target: &Path,
    dry_run: bool,
    allow_degraded: bool,
    backup_tag: &str,
) -> std::io::Result<(bool, u64)> {
    if source == target {
        return Ok((false, 0));
    }
    if !is_safe_path(source) || !is_safe_path(target) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "unsafe path",
        ));
    }

    if let Some(parent) = target.parent() {
        let _dirfd = open_dir_nofollow(parent)?; // RAII drop closes
    }

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

    if dry_run {
        return Ok((false, 0));
    }

    if is_symlink {
        let desired = fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
        let mut resolved_current = current_dest.clone().unwrap_or_default();
        if resolved_current.is_relative() {
            if let Some(parent) = target.parent() {
                resolved_current = parent.join(resolved_current);
            }
        }
        let resolved_current = fs::canonicalize(&resolved_current).unwrap_or(resolved_current);
        if resolved_current == desired {
            return Ok((false, 0));
        }

        // Backup symlink by creating a symlink backup pointing to the same destination
        if let Some(curr) = current_dest.as_ref() {
            let backup = backup_path_with_tag(target, backup_tag);
            let _ = fs::remove_file(&backup);
            let t0 = Instant::now();
            let _ = unix_fs::symlink(curr, &backup);
            // Write sidecar before mutation
            let sc = BackupSidecar {
                schema: "backup_meta.v1".to_string(),
                prior_kind: "symlink".to_string(),
                prior_dest: Some(curr.display().to_string()),
                mode: None,
            };
            let _ = write_sidecar(&backup, &sc);
            let _elapsed_ms = t0.elapsed().as_millis() as u64;
            let _ = _elapsed_ms; // reserved for future telemetry
        }
        // Atomically swap: ensure target removed via cap-handle
        if let Some(parent) = target.parent() {
            let dirfd = open_dir_nofollow(parent)?;
            let fname = target
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("target");
            let fname_c = std::ffi::CString::new(fname).unwrap();
            let _ = unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty());
        }
        let res = atomic_symlink_swap(source, target, allow_degraded)?;
        return Ok(res);
    }

    // Regular file: backup then replace with symlink
    if existed {
        if let Ok(ref meta) = metadata {
            let t0 = Instant::now();
            let parent = target.parent().unwrap_or_else(|| Path::new("."));
            let dirfd = open_dir_nofollow(parent)?;
            let fname = target
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("target");
            let backup_pb = backup_path_with_tag(target, backup_tag);
            let bname = backup_pb
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("backup");
            let fname_c = std::ffi::CString::new(fname).unwrap();
            let bname_c = std::ffi::CString::new(bname).unwrap();
            // Remove any preexisting backup
            let _ = unlinkat(&dirfd, bname_c.as_c_str(), AtFlags::empty());
            // Open source (target) for reading
            let srcfd = openat(&dirfd, fname_c.as_c_str(), OFlags::RDONLY, Mode::empty())
                .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
            // Open destination (backup) for create/truncate
            let dstfd = openat(
                &dirfd,
                bname_c.as_c_str(),
                OFlags::WRONLY | OFlags::CREATE | OFlags::TRUNC,
                Mode::from_bits_truncate(0o600),
            )
            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
            // Copy bytes
            let mut sfile = std::fs::File::from(srcfd);
            let mut dfile = std::fs::File::from(dstfd);
            std::io::copy(&mut sfile, &mut dfile)?;
            // Set permissions on backup to match source meta
            let mode = meta.permissions().mode();
            let dstfd2 = openat(&dirfd, bname_c.as_c_str(), OFlags::RDONLY, Mode::empty())
                .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
            fchmod(&dstfd2, Mode::from_bits_truncate(mode))
                .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
            // Write sidecar before mutation
            let sc = BackupSidecar {
                schema: "backup_meta.v1".to_string(),
                prior_kind: "file".to_string(),
                prior_dest: None,
                mode: Some(format!("{:o}", mode)),
            };
            let _ = write_sidecar(&backup_pb, &sc);
            // Remove original target (will be replaced by atomic swap below)
            unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty())
                .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
            let _elapsed_ms = t0.elapsed().as_millis() as u64;
            let _ = _elapsed_ms; // reserved for future telemetry
        }
    } else {
        // Target did not exist: create a tombstone backup and sidecar with prior_kind=none
        let backup = backup_path_with_tag(target, backup_tag);
        let _ = fs::remove_file(&backup);
        // Create zero-length file as tombstone (best-effort)
        let _ = std::fs::File::create(&backup);
        let sc = BackupSidecar {
            schema: "backup_meta.v1".to_string(),
            prior_kind: "none".to_string(),
            prior_dest: None,
            mode: None,
        };
        let _ = write_sidecar(&backup, &sc);
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    // Ensure target removed via capability handle
    if let Some(parent) = target.parent() {
        let dirfd = open_dir_nofollow(parent)?;
        let fname = target
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("target");
        let fname_c = std::ffi::CString::new(fname).unwrap();
        let _ = unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty());
    }
    let res = atomic_symlink_swap(source, target, allow_degraded)?;
    Ok(res)
}

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
                let old_c = std::ffi::CString::new(bname).unwrap();
                let new_c = std::ffi::CString::new(fname).unwrap();
                renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
                    .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
                // Restore mode when present
                if let Some(ms) = side.mode.as_ref() {
                    if let Ok(m) = u32::from_str_radix(ms, 8) {
                        let fname_c = std::ffi::CString::new(fname).unwrap();
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
                        let old_c = std::ffi::CString::new(bname).unwrap();
                        let new_c = std::ffi::CString::new(fname).unwrap();
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
                    let fname_c = std::ffi::CString::new(fname).unwrap();
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
                    let old_c = std::ffi::CString::new(bname).unwrap();
                    let new_c = std::ffi::CString::new(fname).unwrap();
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
        let old_c = std::ffi::CString::new(bname).unwrap();
        let new_c = std::ffi::CString::new(fname).unwrap();
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
