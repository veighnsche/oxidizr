use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use rustix::fs::{fchmod, fsync, openat, renameat, unlinkat, AtFlags, Mode, OFlags};

use crate::fs::atomic::open_dir_nofollow;

/// Legacy rename of a backup payload into the target place. Removes target first.
///
/// # Errors
///
/// Returns an IO error if the rename operation fails.
pub fn legacy_rename(target_path: &Path, backup: &Path) -> std::io::Result<()> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
    let fname_os = target_path.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target_path has no file name",
        )
    })?;
    let bname_os = backup.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "backup has no file name")
    })?;
    let _ = std::fs::remove_file(target_path);
    let dirfd = open_dir_nofollow(parent)?;
    let old_c = std::ffi::CString::new(bname_os.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
    let new_c = std::ffi::CString::new(fname_os.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
    renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
        .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
    let _ = fsync(&dirfd);
    Ok(())
}

/// Restore file bytes by renaming the backup payload and restoring mode if provided.
///
/// # Errors
///
/// Returns an IO error if the restore operation fails.
pub fn restore_file_bytes(
    target_path: &Path,
    backup: &Path,
    mode_octal: Option<u32>,
) -> std::io::Result<()> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
    let fname_os = target_path.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target_path has no file name",
        )
    })?;
    let bname_os = backup.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "backup has no file name")
    })?;
    let dirfd = open_dir_nofollow(parent)?;
    let _ = std::fs::remove_file(target_path);
    let old_c = std::ffi::CString::new(bname_os.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
    let new_c = std::ffi::CString::new(fname_os.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
    renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
        .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
    if let Some(m) = mode_octal {
        let fname_c = std::ffi::CString::new(fname_os.as_bytes()).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
        })?;
        let tfd = openat(&dirfd, fname_c.as_c_str(), OFlags::RDONLY, Mode::empty())
            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
        let _ = fchmod(&tfd, Mode::from_bits_truncate(m));
    }
    let _ = fsync(&dirfd);
    Ok(())
}

/// Ensure target is absent by unlinking in a TOCTOU-safe way.
///
/// # Errors
///
/// Returns an IO error if the unlink operation fails.
pub fn ensure_absent(target_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = target_path.parent() {
        let dirfd = open_dir_nofollow(parent)?;
        let fname_os = target_path.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "target_path has no file name",
            )
        })?;
        let fname_c = std::ffi::CString::new(fname_os.as_bytes()).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
        })?;
        match unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty()) {
            Ok(()) => {}
            Err(e) if e == rustix::io::Errno::NOENT => {}
            Err(e) => return Err(std::io::Error::from_raw_os_error(e.raw_os_error())),
        }
        let _ = fsync(&dirfd);
        Ok(())
    } else {
        let _ = std::fs::remove_file(target_path);
        Ok(())
    }
}

/// Restore symlink to a destination path atomically.
///
/// # Errors
///
/// Returns an IO error if the symlink restoration fails.
pub fn restore_symlink_to(target_path: &Path, dest: &Path) -> std::io::Result<()> {
    let _ = crate::fs::atomic::atomic_symlink_swap(dest, target_path, true, None)?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn td() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn legacy_rename_moves_backup_to_target() {
        let t = td();
        let root = t.path();
        let tgt = root.join("file.txt");
        let bak = root.join(".file.txt.tag.1.bak");
        std::fs::write(&bak, b"hello").unwrap();
        legacy_rename(&tgt, &bak).unwrap();
        assert!(tgt.exists());
        assert!(!bak.exists());
    }

    #[test]
    fn restore_file_bytes_sets_mode_when_present() {
        use std::os::unix::fs::PermissionsExt;
        let t = td();
        let root = t.path();
        let tgt = root.join("file.txt");
        let bak = root.join(".file.txt.tag.1.bak");
        std::fs::write(&bak, b"hi").unwrap();
        restore_file_bytes(&tgt, &bak, Some(0o600)).unwrap();
        let mode = std::fs::metadata(&tgt).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn ensure_absent_removes_target() {
        let t = td();
        let root = t.path();
        let tgt = root.join("missing");
        std::fs::write(&tgt, b"x").unwrap();
        ensure_absent(&tgt).unwrap();
        assert!(!tgt.exists());
    }

    #[test]
    fn restore_symlink_to_creates_symlink() {
        let t = td();
        let root = t.path();
        let tgt = root.join("usr/bin/app");
        std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
        let dest = root.join("bin");
        std::fs::create_dir_all(&dest).unwrap();
        restore_symlink_to(&tgt, &dest).unwrap();
        assert!(std::fs::symlink_metadata(&tgt)
            .unwrap()
            .file_type()
            .is_symlink());
        let link = std::fs::read_link(&tgt).unwrap();
        // atomic_symlink_swap uses absolute dest
        assert!(link.ends_with("bin"));
    }
}
