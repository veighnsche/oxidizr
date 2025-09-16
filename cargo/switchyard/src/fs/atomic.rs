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

    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let fname = target.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target must not end with a slash",
        )
    })?;

    let pid = std::process::id();
    let ctr = NEXT_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_name = format!(".{}.{}.{}{}", fname.to_string_lossy(), pid, ctr, TMP_SUFFIX);

    let dirfd = open_dir_nofollow(parent)?;

    // Build CStrings once
    let tmp_c = std::ffi::CString::new(tmp_name.as_str()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid tmp cstring")
    })?;
    let new_c = std::ffi::CString::new(fname.as_bytes()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid target name")
    })?;
    let src_c = std::ffi::CString::new(source.as_os_str().as_bytes()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid source path")
    })?;

    // Best-effort cleanup of stray tmp
    match unlinkat(&dirfd, tmp_c.as_c_str(), AtFlags::empty()) {
        Ok(()) | Err(Errno::NOENT) => {}
        Err(e) => return Err(errno_to_io(e)),
    }

    // Create tmp symlink
    symlinkat(src_c.as_c_str(), &dirfd, tmp_c.as_c_str()).map_err(errno_to_io)?;

    // Attempt atomic rename
    let rename_res = renameat(&dirfd, tmp_c.as_c_str(), &dirfd, new_c.as_c_str());

    // Test injection gate
    let allow_env_overrides = std::env::var_os("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES")
        == Some(std::ffi::OsString::from("1"));
    let inject_exdev = match force_exdev {
        Some(b) => b,
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
            // Measure true fsync duration only
            let t_fsync = Instant::now();
            let res = fsync_dirfd(&dirfd);
            let fsync_ms = u64::try_from(t_fsync.elapsed().as_millis()).unwrap_or(u64::MAX);

            if let Err(e) = res {
                // Optional: decide if you want to surface or log; here we return best-effort success.
                // return Err(e); // <-- flip this if fsync must be strict
                let _ = e;
            }

            Ok((false, fsync_ms))
        }
        Err(e) if e == Errno::XDEV && allow_degraded => {
            // Non-atomic: remove final, place new symlink
            match unlinkat(&dirfd, new_c.as_c_str(), AtFlags::empty()) {
                Ok(()) | Err(Errno::NOENT) => {}
                Err(e) => {
                    // Cleanup tmp before returning
                    let _ = unlinkat(&dirfd, tmp_c.as_c_str(), AtFlags::empty());
                    return Err(errno_to_io(e));
                }
            }
            if let Err(e) =
                symlinkat(src_c.as_c_str(), &dirfd, new_c.as_c_str()).map_err(errno_to_io)
            {
                let _ = unlinkat(&dirfd, tmp_c.as_c_str(), AtFlags::empty());
                return Err(e);
            }

            // We no longer need tmp; best-effort cleanup
            let _ = unlinkat(&dirfd, tmp_c.as_c_str(), AtFlags::empty());

            let t_fsync = Instant::now();
            let res = fsync_dirfd(&dirfd);
            let fsync_ms = u64::try_from(t_fsync.elapsed().as_millis()).unwrap_or(u64::MAX);
            if let Err(e) = res {
                let _ = e;
            }
            Ok((true, fsync_ms))
        }
        Err(e) => {
            // General failure: best-effort cleanup of tmp
            let _ = unlinkat(&dirfd, tmp_c.as_c_str(), AtFlags::empty());
            Err(errno_to_io(e))
        }
    }
}
