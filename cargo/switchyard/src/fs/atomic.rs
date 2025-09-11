/// placeholder

use std::fs;
use std::os::unix::io::RawFd;
use std::path::Path;
use libc;

pub fn open_dir_nofollow(dir: &Path) -> std::io::Result<RawFd> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(dir.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid path"))?;
    let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW;
    let fd = unsafe { libc::open(c.as_ptr(), flags, 0) };
    if fd < 0 { return Err(std::io::Error::last_os_error()); }
    Ok(fd)
}

pub fn fsync_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let dir = fs::File::open(parent)?;
        dir.sync_all()?;
    }
    Ok(())
}

pub fn atomic_symlink_swap(source: &Path, target: &Path, allow_degraded: bool) -> std::io::Result<bool> {
    // Open parent directory with O_DIRECTORY | O_NOFOLLOW to prevent traversal and races
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
    let tmp_name = format!(".{}.oxidizr.tmp", fname);

    let dirfd = open_dir_nofollow(parent)?;

    // Best-effort unlink temporary name if present (ignore errors)
    let tmp_c = std::ffi::CString::new(tmp_name.as_str()).unwrap();
    unsafe { libc::unlinkat(dirfd, tmp_c.as_ptr(), 0) };

    // Create symlink using symlinkat relative to parent dirfd
    // Note: symlink "target" argument is the link's content (where it points to)
    use std::os::unix::ffi::OsStrExt;
    let src_c = std::ffi::CString::new(source.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid source path"))?;
    let tmp_c2 = std::ffi::CString::new(tmp_name.as_str()).unwrap();
    let rc_symlink = unsafe { libc::symlinkat(src_c.as_ptr(), dirfd, tmp_c2.as_ptr()) };
    if rc_symlink != 0 {
        let e = std::io::Error::last_os_error();
        unsafe { libc::close(dirfd) };
        return Err(e);
    }

    // Atomically rename tmp -> fname within the same directory
    let new_c = std::ffi::CString::new(fname).unwrap();
    let rc = unsafe { libc::renameat(dirfd, tmp_c2.as_ptr(), dirfd, new_c.as_ptr()) };
    if rc != 0 {
        let last = std::io::Error::last_os_error();
        if last.raw_os_error() == Some(libc::EXDEV) && allow_degraded {
            // Fall back: best-effort non-atomic replacement
            // Remove final if present
            unsafe { libc::unlinkat(dirfd, new_c.as_ptr(), 0) };
            // Create symlink directly at final name
            let rc2 = unsafe { libc::symlinkat(src_c.as_ptr(), dirfd, new_c.as_ptr()) };
            let last2 = std::io::Error::last_os_error();
            unsafe { libc::close(dirfd) };
            if rc2 != 0 { return Err(last2); }
            let _ = fsync_parent_dir(target);
            return Ok(true);
        } else {
            unsafe { libc::close(dirfd) };
            return Err(last);
        }
    }
    let _ = fsync_parent_dir(target);
    unsafe { libc::close(dirfd) };
    Ok(false)
}
