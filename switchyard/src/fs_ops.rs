use std::fs;
use std::os::unix::fs as unix_fs;
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

pub fn atomic_symlink_swap(source: &Path, target: &Path) -> std::io::Result<()> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let fname = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
    let tmp_name = format!(".{}.oxidizr.tmp", fname);
    let tmp = parent.join(&tmp_name);
    let _ = fs::remove_file(&tmp);
    unix_fs::symlink(source, &tmp)?;

    let dirfd = open_dir_nofollow(parent)?;
    let old_c = std::ffi::CString::new(tmp_name.as_str()).unwrap();
    let new_c = std::ffi::CString::new(fname).unwrap();
    let rc = unsafe { libc::renameat(dirfd, old_c.as_ptr(), dirfd, new_c.as_ptr()) };
    let last = std::io::Error::last_os_error();
    unsafe { libc::close(dirfd) };
    if rc != 0 { return Err(last); }

    let _ = fsync_parent_dir(target);
    Ok(())
}
