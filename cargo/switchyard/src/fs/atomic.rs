use std::fs;
use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{openat, renameat, symlinkat, unlinkat, AtFlags, Mode, OFlags, CWD};
use rustix::io::Errno;
use std::time::Instant;

fn errno_to_io(e: Errno) -> std::io::Error {
    std::io::Error::from_raw_os_error(e.raw_os_error())
}

pub fn open_dir_nofollow(dir: &Path) -> std::io::Result<OwnedFd> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(dir.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid path"))?;
    openat(
        &CWD,
        c.as_c_str(),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(errno_to_io)
}

pub fn fsync_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let dir = fs::File::open(parent)?;
        dir.sync_all()?;
    }
    Ok(())
}

pub fn atomic_symlink_swap(source: &Path, target: &Path, allow_degraded: bool) -> std::io::Result<(bool, u64)> {
    // Open parent directory with O_DIRECTORY | O_NOFOLLOW to prevent traversal and races
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
    let tmp_name = format!(".{}.oxidizr.tmp", fname);

    let dirfd = open_dir_nofollow(parent)?;

    // Best-effort unlink temporary name if present (ignore errors)
    let tmp_c = std::ffi::CString::new(tmp_name.as_str()).unwrap();
    let _ = unlinkat(&dirfd, tmp_c.as_c_str(), AtFlags::empty());

    // Create symlink using symlinkat relative to parent dirfd
    use std::os::unix::ffi::OsStrExt;
    let src_c = std::ffi::CString::new(source.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid source path"))?;
    let tmp_c2 = std::ffi::CString::new(tmp_name.as_str()).unwrap();
    symlinkat(src_c.as_c_str(), &dirfd, tmp_c2.as_c_str()).map_err(errno_to_io)?;

    // Atomically rename tmp -> fname within the same directory
    let new_c = std::ffi::CString::new(fname).unwrap();
    let t0 = Instant::now();
    match renameat(&dirfd, tmp_c2.as_c_str(), &dirfd, new_c.as_c_str()) {
        Ok(()) => {
            let _ = fsync_parent_dir(target);
            let fsync_ms = t0.elapsed().as_millis() as u64;
            Ok((false, fsync_ms))
        }
        Err(e) if e == Errno::XDEV && allow_degraded => {
            // Fall back: best-effort non-atomic replacement
            let _ = unlinkat(&dirfd, new_c.as_c_str(), AtFlags::empty());
            symlinkat(src_c.as_c_str(), &dirfd, new_c.as_c_str()).map_err(errno_to_io)?;
            let _ = fsync_parent_dir(target);
            let fsync_ms = t0.elapsed().as_millis() as u64;
            Ok((true, fsync_ms))
        }
        Err(e) => Err(errno_to_io(e)),
    }
}
