use crate::{Error, Result};
use std::fs;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

fn mount_entry_for(path: &Path) -> Option<(PathBuf, String)> {
    // Parse /proc/self/mounts and select the longest mountpoint that prefixes `path`.
    let mut f = match fs::File::open("/proc/self/mounts") {
        Ok(f) => f,
        Err(_) => return None,
    };
    let mut s = String::new();
    if f.read_to_string(&mut s).is_err() {
        return None;
    }
    let p = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };
    let mut best: Option<(PathBuf, String)> = None;
    for line in s.lines() {
        // format: <src> <mountpoint> <fstype> <opts> ...
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let mnt = PathBuf::from(parts[1]);
        if p.starts_with(&mnt) {
            let opts = parts[3].to_string();
            match &best {
                None => best = Some((mnt, opts)),
                Some((b, _)) => {
                    if mnt.as_os_str().len() > b.as_os_str().len() {
                        best = Some((mnt, opts));
                    }
                }
            }
        }
    }
    best
}

pub fn ensure_mount_rw_exec(path: &Path) -> Result<()> {
    if let Some((_mnt, opts)) = mount_entry_for(path) {
        let opts_l = opts.to_ascii_lowercase();
        let has_rw = opts_l.split(',').any(|o| o == "rw");
        let noexec = opts_l.split(',').any(|o| o == "noexec");
        if !has_rw || noexec {
            return Err(Error::ExecutionFailed(format!(
                "Filesystem at '{}' not suitable: requires rw and exec (opts: {})",
                path.display(), opts
            )));
        }
    }
    Ok(())
}

pub fn check_immutable(path: &Path) -> Result<()> {
    // Best-effort: if lsattr available, check immutable flag on the path itself (not recursive).
    let out = std::process::Command::new("lsattr")
        .args(["-d", path.as_os_str().to_string_lossy().as_ref()])
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Output example: "----i-------- /usr/bin/sudo"
            for line in stdout.lines() {
                let mut fields = line.split_whitespace();
                if let Some(attrs) = fields.next() {
                    if attrs.contains('i') {
                        return Err(Error::ExecutionFailed(format!(
                            "Target '{}' is immutable (chattr +i). Run 'chattr -i {}' to clear before proceeding.",
                            path.display(), path.display()
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn check_source_trust(source: &Path, force: bool) -> Result<()> {
    let meta = fs::symlink_metadata(source)?;
    // Ensure not world-writable
    let mode = meta.mode();
    if (mode & 0o002) != 0 {
        if force {
            tracing::warn!("source_trust: world-writable source {} allowed due to --force", source.display());
        } else {
            return Err(Error::ExecutionFailed(format!(
                "Untrusted source (world-writable): {}. Pass --force to override.",
                source.display()
            )));
        }
    }
    // Ensure owned by root
    if meta.uid() != 0 {
        if force {
            tracing::warn!("source_trust: non-root-owned source {} allowed due to --force", source.display());
        } else {
            return Err(Error::ExecutionFailed(format!(
                "Untrusted source (not root-owned): {}. Pass --force to override.",
                source.display()
            )));
        }
    }
    // Ensure on an exec mount
    ensure_mount_rw_exec(source)?;
    // Disallow sources under HOME unless forced
    if let Ok(home) = std::env::var("HOME") {
        let home_p = Path::new(&home);
        if source.starts_with(home_p) && !force {
            return Err(Error::ExecutionFailed(format!(
                "Untrusted source under HOME: {}. Pass --force to override.",
                source.display()
            )));
        }
    }
    Ok(())
}
