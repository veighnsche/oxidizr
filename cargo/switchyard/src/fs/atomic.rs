//! Atomic symlink swap primitives and helpers.
//!
//! This module implements a TOCTOU-safe sequence using directory handles:
//! `open_dir_nofollow(parent) -> symlinkat(tmp) -> renameat(tmp, final) -> fsync(dirfd)`.
//!
//! Test override knobs:
//! - `SWITCHYARD_FORCE_EXDEV=1` — simulate a cross-filesystem rename error (EXDEV) to exercise
//!   degraded fallback paths and telemetry in higher layers.
use std::fs;
use std::path::Path;

use crate::constants::TMP_SUFFIX;
use rustix::fd::OwnedFd;
use rustix::fs::{openat, renameat, symlinkat, unlinkat, AtFlags, Mode, OFlags, CWD};
use rustix::io::Errno;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

fn errno_to_io(e: Errno) -> std::io::Error {
    std::io::Error::from_raw_os_error(e.raw_os_error())
}

// Global counter to produce unique temporary names within a process.
static NEXT_TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Open a directory with `O_DIRECTORY` | `O_NOFOLLOW` for atomic operations.
///
/// # Errors
///
/// Returns an IO error if the directory cannot be opened.
pub fn open_dir_nofollow(dir: &Path) -> std::io::Result<OwnedFd> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(dir.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid path"))?;
    openat(
        CWD,
        c.as_c_str(),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(errno_to_io)
}

/// Fsync the parent directory of `path` for durability.
///
/// # Errors
///
/// Returns an IO error if the parent directory cannot be opened or fsynced.
pub fn fsync_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let dir = fs::File::open(parent)?;
        dir.sync_all()?;
    }
    Ok(())
}

/// Fsync a directory using an already-open directory file descriptor.
///
/// This avoids a TOCTOU window from re-opening the directory by path.
fn fsync_dirfd(dirfd: &OwnedFd) -> std::io::Result<()> {
    rustix::fs::fsync(dirfd).map_err(errno_to_io)
}

/// Atomically swap a symlink target using a temporary file and renameat.
///
/// # Errors
///
/// Returns an IO error if the atomic swap operation fails.
pub fn atomic_symlink_swap(
    source: &Path,
    target: &Path,
    allow_degraded: bool,
    force_exdev: Option<bool>,
) -> std::io::Result<(bool, u64)> {
    use std::os::unix::ffi::OsStrExt;
    // Open parent directory with O_DIRECTORY | O_NOFOLLOW to prevent traversal and races
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let fname = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("target");
    let pid = std::process::id();
    let ctr = NEXT_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_name = format!(".{fname}.{pid}.{ctr}{TMP_SUFFIX}");

    let dirfd = open_dir_nofollow(parent)?;

    // Best-effort unlink temporary name if present (ignore ENOENT only)
    let tmp_c = std::ffi::CString::new(tmp_name.as_str())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
    match unlinkat(&dirfd, tmp_c.as_c_str(), AtFlags::empty()) {
        Ok(()) => {}
        Err(e) if e == Errno::NOENT => {}
        Err(e) => return Err(errno_to_io(e)),
    }

    // Create symlink using symlinkat relative to parent dirfd
    let src_c = std::ffi::CString::new(source.as_os_str().as_bytes()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid source path")
    })?;
    let tmp_c2 = std::ffi::CString::new(tmp_name.as_str())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
    symlinkat(src_c.as_c_str(), &dirfd, tmp_c2.as_c_str()).map_err(errno_to_io)?;

    // Atomically rename tmp -> final name within the same directory (bytes-safe)
    let new_c = if let Some(name_os) = target.file_name() {
        std::ffi::CString::new(name_os.as_bytes())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?
    } else {
        std::ffi::CString::new("target")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?
    };
    let t0 = Instant::now();
    let rename_res = renameat(&dirfd, tmp_c2.as_c_str(), &dirfd, new_c.as_c_str());
    // Prefer per-instance override when provided; fallback to gated env for legacy tests only.
    // Note: do not enable env overrides by default in tests to avoid cross-scenario interference.
    let allow_env_overrides =
        std::env::var_os("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES") == Some(std::ffi::OsString::from("1"));
    let inject_exdev = match force_exdev {
        Some(true) => true,
        Some(false) => false,
        None => {
            allow_env_overrides
                && std::env::var_os("SWITCHYARD_FORCE_EXDEV") == Some(std::ffi::OsString::from("1"))
        }
    };
    let rename_res = if inject_exdev {
        match rename_res {
            Ok(()) => Err(Errno::XDEV),
            Err(e) => Err(e),
        }
    } else {
        rename_res
    };
    match rename_res {
        Ok(()) => {
            let _ = fsync_dirfd(&dirfd);
            let fsync_ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
            Ok((false, fsync_ms))
        }
        Err(e) if e == Errno::XDEV && allow_degraded => {
            // Fall back: best-effort non-atomic replacement
            match unlinkat(&dirfd, new_c.as_c_str(), AtFlags::empty()) {
                Ok(()) => {}
                Err(e) if e == Errno::NOENT => {}
                Err(e) => return Err(errno_to_io(e)),
            }
            symlinkat(src_c.as_c_str(), &dirfd, new_c.as_c_str()).map_err(errno_to_io)?;
            let _ = fsync_dirfd(&dirfd);
            let fsync_ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
            Ok((true, fsync_ms))
        }
        Err(e) => Err(errno_to_io(e)),
    }
}
