//! Backup and sidecar helpers for Switchyard filesystem operations.
//!
//! This module centralizes the backup payload and sidecar schema handling used
//! by symlink replacement and restore operations.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Generate a unique backup path for a target file (includes a timestamp).
/// Public so callers (preflight/tests) can compute expected names.
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

// Return the previous (second newest) backup pair if present.
pub(crate) fn find_previous_backup_and_sidecar(
    target: &Path,
    tag: &str,
) -> Option<(Option<PathBuf>, PathBuf)> {
    let name = target.file_name()?.to_str()?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{}.{}.", name, tag);
    let mut stamps: Vec<(u128, PathBuf)> = Vec::new();
    let mut seen: std::collections::HashSet<u128> = std::collections::HashSet::new();
    if let Ok(rd) = std::fs::read_dir(parent) {
        for e in rd.flatten() {
            let fname = e.file_name();
            if let Some(s) = fname.to_str() {
                if let Some(rest) = s.strip_prefix(&prefix) {
                    if let Some(num_s) = rest
                        .strip_suffix(".bak")
                        .or_else(|| rest.strip_suffix(".bak.meta.json"))
                    {
                        if let Ok(num) = num_s.parse::<u128>() {
                            if seen.insert(num) {
                                // base path ending with .bak
                                let base = parent
                                    .join(format!("{}.bak", prefix.clone() + &num.to_string()));
                                stamps.push((num, base));
                            }
                        }
                    }
                }
            }
        }
    }
    if stamps.len() < 2 {
        return None;
    }
    stamps.sort_by(|a, b| a.0.cmp(&b.0));
    let (_ts, base) = stamps[stamps.len() - 2].clone();
    let sc = sidecar_path_for_backup(&base);
    let backup_present = if base.exists() { Some(base) } else { None };
    Some((backup_present, sc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_BACKUP_TAG;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn snapshot_file_creates_payload_and_sidecar() {
        let t = tmp();
        let root = t.path();
        let tgt = root.join("file.txt");
        std::fs::write(&tgt, b"hello").unwrap();
        create_snapshot(&tgt, DEFAULT_BACKUP_TAG).unwrap();
        let pair = find_latest_backup_and_sidecar(&tgt, DEFAULT_BACKUP_TAG).expect("pair");
        assert!(pair.0.is_some(), "payload present");
        assert!(pair.1.exists(), "sidecar exists");
    }

    #[test]
    fn snapshot_symlink_creates_symlink_backup_and_sidecar() {
        let t = tmp();
        let root = t.path();
        let target = root.join("bin");
        std::fs::create_dir_all(&target).unwrap();
        let link = root.join("usr/bin/app");
        std::fs::create_dir_all(link.parent().unwrap()).unwrap();
        let _ = std::os::unix::fs::symlink("../../bin", &link); // relative symlink
        create_snapshot(&link, DEFAULT_BACKUP_TAG).unwrap();
        let pair = find_latest_backup_and_sidecar(&link, DEFAULT_BACKUP_TAG).expect("pair");
        assert!(pair.1.exists(), "sidecar exists");
    }

    #[test]
    fn snapshot_none_creates_tombstone_and_sidecar() {
        let t = tmp();
        let root = t.path();
        let tgt = root.join("missing");
        assert!(!tgt.exists());
        create_snapshot(&tgt, DEFAULT_BACKUP_TAG).unwrap();
        let pair = find_latest_backup_and_sidecar(&tgt, DEFAULT_BACKUP_TAG).expect("pair");
        assert!(pair.1.exists(), "sidecar exists");
    }
}

/// Create a snapshot (backup payload and sidecar) of the current target state.
/// - If target is a regular file: copy bytes to a timestamped backup and record mode in sidecar.
/// - If target is a symlink: create a symlink backup pointing to current dest and write sidecar with prior_dest.
/// - If target is absent: create a tombstone payload and sidecar with prior_kind="none".
pub fn create_snapshot(target: &Path, backup_tag: &str) -> std::io::Result<()> {
    use crate::fs::atomic::open_dir_nofollow;
    use rustix::fs::{fchmod, openat, AtFlags, Mode, OFlags};
    use std::fs;
    use std::os::unix;
    use std::os::unix::fs::PermissionsExt as _;

    let metadata = fs::symlink_metadata(target);
    let existed = metadata.is_ok();
    let is_symlink = metadata
        .as_ref()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);

    if is_symlink {
        let current_dest = fs::read_link(target).ok();
        if let Some(curr) = current_dest.as_ref() {
            let backup = backup_path_with_tag(target, backup_tag);
            let _ = fs::remove_file(&backup);
            // Create a symlink backup pointing to the same destination
            let _ = unix::fs::symlink(curr, &backup);
            // Write sidecar
            let sc = BackupSidecar {
                schema: "backup_meta.v1".to_string(),
                prior_kind: "symlink".to_string(),
                prior_dest: Some(curr.display().to_string()),
                mode: None,
            };
            write_sidecar(&backup, &sc).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("sidecar write failed: {}", e),
                )
            })?;
        }
        return Ok(());
    }

    if existed {
        if let Ok(ref meta) = metadata {
            // Copy to backup within the same directory using TOCTOU-safe fds
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
            let fname_c = std::ffi::CString::new(fname).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
            })?;
            let bname_c = std::ffi::CString::new(bname).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
            })?;
            // Remove any preexisting backup file
            let _ = rustix::fs::unlinkat(&dirfd, bname_c.as_c_str(), AtFlags::empty());
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
            // Write sidecar
            let sc = BackupSidecar {
                schema: "backup_meta.v1".to_string(),
                prior_kind: "file".to_string(),
                prior_dest: None,
                mode: Some(format!("{:o}", mode)),
            };
            write_sidecar(&backup_pb, &sc).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("sidecar write failed: {}", e),
                )
            })?;
            return Ok(());
        }
    }

    // Target did not exist: create a tombstone and sidecar
    let backup = backup_path_with_tag(target, backup_tag);
    let _ = fs::remove_file(&backup);
    let _ = std::fs::File::create(&backup);
    let sc = BackupSidecar {
        schema: "backup_meta.v1".to_string(),
        prior_kind: "none".to_string(),
        prior_dest: None,
        mode: None,
    };
    write_sidecar(&backup, &sc).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("sidecar write failed: {}", e),
        )
    })
}

/// Public helper for preflight/tests: check if there are backup artifacts (payload and/or sidecar)
/// for the given target and tag.
pub fn has_backup_artifacts(target: &Path, tag: &str) -> bool {
    if let Some((payload, sc)) = find_latest_backup_and_sidecar(target, tag) {
        payload.is_some() || sc.exists()
    } else {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BackupSidecar {
    pub(crate) schema: String,     // "backup_meta.v1"
    pub(crate) prior_kind: String, // "file" | "symlink" | "none"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prior_dest: Option<String>, // for symlink
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mode: Option<String>, // octal string for file, e.g. "100644"
}

pub(crate) fn sidecar_path_for_backup(backup: &Path) -> PathBuf {
    let s = backup.as_os_str().to_owned();
    use std::ffi::OsString;
    let mut s2 = OsString::from(s);
    s2.push(".meta.json");
    PathBuf::from(s2)
}

pub(crate) fn write_sidecar(backup: &Path, sc: &BackupSidecar) -> std::io::Result<()> {
    let sc_path = sidecar_path_for_backup(backup);
    if let Some(parent) = sc_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let f = std::fs::File::create(&sc_path)?;
    serde_json::to_writer_pretty(f, sc)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub(crate) fn read_sidecar(sc_path: &Path) -> std::io::Result<BackupSidecar> {
    let f = std::fs::File::open(sc_path)?;
    serde_json::from_reader(f).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

// Return (backup_path_if_present, sidecar_path) for the latest timestamped pair.
pub(crate) fn find_latest_backup_and_sidecar(
    target: &Path,
    tag: &str,
) -> Option<(Option<PathBuf>, PathBuf)> {
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
                    if let Some(num_s) = rest
                        .strip_suffix(".bak")
                        .or_else(|| rest.strip_suffix(".bak.meta.json"))
                    {
                        if let Ok(num) = num_s.parse::<u128>() {
                            if best.as_ref().map(|(b, _)| num > *b).unwrap_or(true) {
                                // compute base path ending with .bak
                                let base = parent
                                    .join(format!("{}.bak", prefix.clone() + &num.to_string()));
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
