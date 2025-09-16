use std::path::Path;

use crate::fs::atomic::open_dir_nofollow;
use rustix::fs::{fchmod, openat, AtFlags, Mode, OFlags};
use std::fs;
use std::os::unix;
use std::os::unix::fs::PermissionsExt as _;

use super::sidecar::{write_sidecar, BackupSidecar};

/// Generate a unique backup path for a target file (includes a timestamp).
/// Public so callers (preflight/tests) can compute expected names.
#[must_use]
pub fn backup_path_with_tag(target: &Path, tag: &str) -> std::path::PathBuf {
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
    parent.join(format!(".{name}.{tag}.{ts}.bak"))
}

/// Create a snapshot (backup payload and sidecar) of the current target state.
///
/// # Errors
///
/// Returns an IO error if the snapshot creation fails.
/// - If target is a regular file: copy bytes to a timestamped backup and record mode in sidecar.
/// - If target is a symlink: create a symlink backup pointing to current dest and write sidecar with `prior_dest`.
/// - If target is absent: create a tombstone payload and sidecar with `prior_kind="none"`.
pub fn create_snapshot(target: &Path, backup_tag: &str) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(target);
    let existed = metadata.is_ok();
    let is_symlink = metadata
        .as_ref()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);

    if is_symlink {
        let current_dest = fs::read_link(target).ok();
        if let Some(curr) = current_dest.as_ref() {
            // Compute a unique backup path for this instant
            let mut backup = backup_path_with_tag(target, backup_tag);
            while backup.exists() {
                // Bump the numeric timestamp suffix until unique
                if let Some(stem) = backup.file_name().and_then(|s| s.to_str()) {
                    if let Some(prefix) = stem.strip_suffix(".bak") {
                        if let Some((pre, ts_s)) = prefix.rsplit_once('.') {
                            if let Ok(ts) = ts_s.parse::<u128>() {
                                let bumped = format!("{pre}.{}.bak", ts.saturating_add(1));
                                backup = backup
                                    .parent()
                                    .unwrap_or_else(|| Path::new("."))
                                    .join(bumped);
                                continue;
                            }
                        }
                    }
                }
                // Fallback: break to avoid infinite loop
                break;
            }
            let _ = fs::remove_file(&backup);
            // Create a symlink backup pointing to the same destination
            let _ = unix::fs::symlink(curr, &backup);
            // Write sidecar
            let sc = BackupSidecar {
                schema: "backup_meta.v1".to_string(),
                prior_kind: "symlink".to_string(),
                prior_dest: Some(curr.display().to_string()),
                mode: None,
                payload_hash: None,
            };
            write_sidecar(&backup, &sc)
                .map_err(|e| std::io::Error::other(format!("sidecar write failed: {e}")))?;
            // Durability: best-effort parent fsync
            let _ = crate::fs::atomic::fsync_parent_dir(target);
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
            // Compute a unique backup path for this instant
            let mut backup_pb = backup_path_with_tag(target, backup_tag);
            while backup_pb.exists() {
                if let Some(stem) = backup_pb.file_name().and_then(|s| s.to_str()) {
                    if let Some(prefix) = stem.strip_suffix(".bak") {
                        if let Some((pre, ts_s)) = prefix.rsplit_once('.') {
                            if let Ok(ts) = ts_s.parse::<u128>() {
                                let bumped = format!("{pre}.{}.bak", ts.saturating_add(1));
                                backup_pb = backup_pb
                                    .parent()
                                    .unwrap_or_else(|| Path::new("."))
                                    .join(bumped);
                                continue;
                            }
                        }
                    }
                }
                break;
            }
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
            let mut sfile = fs::File::from(srcfd);
            let mut dfile = fs::File::from(dstfd);
            std::io::copy(&mut sfile, &mut dfile)?;
            // Set permissions on backup to match source meta
            let mode = meta.permissions().mode();
            let dstfd2 = openat(&dirfd, bname_c.as_c_str(), OFlags::RDONLY, Mode::empty())
                .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
            fchmod(&dstfd2, Mode::from_bits_truncate(mode))
                .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
            // Compute payload hash for sidecar v2
            let payload_hash = crate::fs::meta::sha256_hex_of(&backup_pb);
            // Sync backup file for durability
            let _ = dfile.sync_all();
            // Write sidecar
            let sc = BackupSidecar {
                schema: if payload_hash.is_some() {
                    "backup_meta.v2".to_string()
                } else {
                    "backup_meta.v1".to_string()
                },
                prior_kind: "file".to_string(),
                prior_dest: None,
                mode: Some(format!("{mode:o}")),
                payload_hash,
            };
            write_sidecar(&backup_pb, &sc)
                .map_err(|e| std::io::Error::other(format!("sidecar write failed: {e}")))?;
            // Durability: ensure parent dir sync
            let _ = crate::fs::atomic::fsync_parent_dir(target);
            return Ok(());
        }
    }

    // Target did not exist: create a tombstone and sidecar
    let mut backup = backup_path_with_tag(target, backup_tag);
    while backup.exists() {
        if let Some(stem) = backup.file_name().and_then(|s| s.to_str()) {
            if let Some(prefix) = stem.strip_suffix(".bak") {
                if let Some((pre, ts_s)) = prefix.rsplit_once('.') {
                    if let Ok(ts) = ts_s.parse::<u128>() {
                        let bumped = format!("{pre}.{}.bak", ts.saturating_add(1));
                        backup = backup
                            .parent()
                            .unwrap_or_else(|| Path::new("."))
                            .join(bumped);
                        continue;
                    }
                }
            }
        }
        break;
    }
    let _ = fs::remove_file(&backup);
    let f = fs::File::create(&backup)?;
    let _ = f.sync_all();
    let sc = BackupSidecar {
        schema: "backup_meta.v1".to_string(),
        prior_kind: "none".to_string(),
        prior_dest: None,
        mode: None,
        payload_hash: None,
    };
    write_sidecar(&backup, &sc)
        .map_err(|e| std::io::Error::other(format!("sidecar write failed: {e}")))?;
    // Durability: parent dir sync
    let _ = crate::fs::atomic::fsync_parent_dir(target);
    Ok(())
}

/// Public helper for preflight/tests: check if there are backup artifacts (payload and/or sidecar)
/// for the given target and tag.
#[must_use]
pub fn has_backup_artifacts(target: &Path, tag: &str) -> bool {
    if let Some((payload, sc)) = super::index::find_latest_backup_and_sidecar(target, tag) {
        payload.is_some() || sc.exists()
    } else {
        false
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_BACKUP_TAG;
    use crate::fs::backup::index;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn snapshot_file_creates_payload_and_sidecar() {
        let t = tmp();
        let root = t.path();
        let tgt = root.join("file.txt");
        fs::write(&tgt, b"hello").unwrap();
        create_snapshot(&tgt, DEFAULT_BACKUP_TAG).unwrap();
        let pair = index::find_latest_backup_and_sidecar(&tgt, DEFAULT_BACKUP_TAG).expect("pair");
        assert!(pair.0.is_some(), "payload present");
        assert!(pair.1.exists(), "sidecar exists");
    }

    #[test]
    fn snapshot_symlink_creates_symlink_backup_and_sidecar() {
        let t = tmp();
        let root = t.path();
        let target = root.join("bin");
        fs::create_dir_all(&target).unwrap();
        let link = root.join("usr/bin/app");
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        let _ = unix::fs::symlink("../../bin", &link); // relative symlink
        create_snapshot(&link, DEFAULT_BACKUP_TAG).unwrap();
        let pair = index::find_latest_backup_and_sidecar(&link, DEFAULT_BACKUP_TAG).expect("pair");
        assert!(pair.1.exists(), "sidecar exists");
    }

    #[test]
    fn snapshot_none_creates_tombstone_and_sidecar() {
        let t = tmp();
        let root = t.path();
        let tgt = root.join("missing");
        assert!(!tgt.exists());
        create_snapshot(&tgt, DEFAULT_BACKUP_TAG).unwrap();
        let pair = index::find_latest_backup_and_sidecar(&tgt, DEFAULT_BACKUP_TAG).expect("pair");
        assert!(pair.1.exists(), "sidecar exists");
    }
}
