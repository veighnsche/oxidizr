use std::path::Path;

use crate::fs::atomic::open_dir_nofollow;
use rustix::fs::{fchmod, openat, AtFlags, Mode, OFlags};
use std::fs;
use std::os::unix;
use std::os::unix::fs::PermissionsExt as _;

use super::index::backup_path_with_tag;
use super::sidecar::{write_sidecar, BackupSidecar};

/// Create a snapshot (backup payload and sidecar) of the current target state.
/// - If target is a regular file: copy bytes to a timestamped backup and record mode in sidecar.
/// - If target is a symlink: create a symlink backup pointing to current dest and write sidecar with prior_dest.
/// - If target is absent: create a tombstone payload and sidecar with prior_kind="none".
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
                payload_hash: None,
            };
            write_sidecar(&backup, &sc).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("sidecar write failed: {}", e),
                )
            })?;
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
            // Compute payload hash for sidecar v2
            let payload_hash = crate::fs::meta::sha256_hex_of(&backup_pb);
            // Sync backup file for durability
            let _ = dfile.sync_all();
            // Write sidecar
            let sc = BackupSidecar {
                schema: if payload_hash.is_some() { "backup_meta.v2".to_string() } else { "backup_meta.v1".to_string() },
                prior_kind: "file".to_string(),
                prior_dest: None,
                mode: Some(format!("{:o}", mode)),
                payload_hash,
            };
            write_sidecar(&backup_pb, &sc).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("sidecar write failed: {}", e),
                )
            })?;
            // Durability: ensure parent dir sync
            let _ = crate::fs::atomic::fsync_parent_dir(target);
            return Ok(());
        }
    }

    // Target did not exist: create a tombstone and sidecar
    let backup = backup_path_with_tag(target, backup_tag);
    let _ = fs::remove_file(&backup);
    let f = std::fs::File::create(&backup)?;
    let _ = f.sync_all();
    let sc = BackupSidecar {
        schema: "backup_meta.v1".to_string(),
        prior_kind: "none".to_string(),
        prior_dest: None,
        mode: None,
        payload_hash: None,
    };
    write_sidecar(&backup, &sc).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("sidecar write failed: {}", e),
        )
    })?;
    // Durability: parent dir sync
    let _ = crate::fs::atomic::fsync_parent_dir(target);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_BACKUP_TAG;
    use crate::fs::backup::sidecar;

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
        let pair = sidecar::find_latest_backup_and_sidecar(&tgt, DEFAULT_BACKUP_TAG).expect("pair");
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
        let pair = sidecar::find_latest_backup_and_sidecar(&link, DEFAULT_BACKUP_TAG).expect("pair");
        assert!(pair.1.exists(), "sidecar exists");
    }

    #[test]
    fn snapshot_none_creates_tombstone_and_sidecar() {
        let t = tmp();
        let root = t.path();
        let tgt = root.join("missing");
        assert!(!tgt.exists());
        create_snapshot(&tgt, DEFAULT_BACKUP_TAG).unwrap();
        let pair = sidecar::find_latest_backup_and_sidecar(&tgt, DEFAULT_BACKUP_TAG).expect("pair");
        assert!(pair.1.exists(), "sidecar exists");
    }
}
