use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Ensure the filesystem backing `path` is read-write and not mounted with noexec.
/// Returns Ok(()) if suitable; Err(String) with a human message otherwise.
pub fn ensure_mount_rw_exec(path: &Path) -> Result<(), String> {
    // Delegate to fs::mount inspector; fail closed on ambiguity.
    match crate::fs::mount::ensure_rw_exec(&crate::fs::mount::ProcStatfsInspector, path) {
        Ok(()) => Ok(()),
        Err(_) => Err(format!(
            "Filesystem at '{}' not suitable or ambiguous (requires rw and exec)",
            path.display()
        )),
    }
}

/// Detect hardlink hazard: returns Ok(true) when the target node has more than one
/// hardlink (nlink > 1). Uses symlink_metadata to avoid following symlinks; callers
/// may optionally resolve and re-check as needed.
pub fn check_hardlink_hazard(path: &Path) -> std::io::Result<bool> {
    if let Ok(md) = std::fs::symlink_metadata(path) {
        // Only consider regular files for this hazard; symlinks/dirs are ignored.
        let ft = md.file_type();
        if ft.is_file() {
            let n = md.nlink();
            return Ok(n > 1);
        }
    }
    Ok(false)
}

/// Best-effort check for SUID/SGID risk on a target path.
/// Returns Ok(true) when either SUID (04000) or SGID (02000) bit is set on the
/// resolved file; Ok(false) otherwise. On errors reading metadata, returns Ok(false)
/// to avoid spurious stops; callers may add an informational note if desired.
pub fn check_suid_sgid_risk(path: &Path) -> std::io::Result<bool> {
    // If path is a symlink, resolve to the destination for inspection.
    let inspect_path = if let Ok(md) = std::fs::symlink_metadata(path) {
        if md.file_type().is_symlink() {
            if let Some(p) = crate::fs::meta::resolve_symlink_target(path) {
                p
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        }
    } else {
        path.to_path_buf()
    };
    if let Ok(meta) = std::fs::metadata(&inspect_path) {
        let mode = meta.mode();
        let risk = (mode & 0o6000) != 0; // SUID (04000) or SGID (02000)
        return Ok(risk);
    }
    Ok(false)
}

/// Best-effort check for the immutable attribute via `lsattr -d`.
/// Returns `Err(String)` only when the target itself is immutable.
/// If `lsattr` is missing or fails, this returns `Ok(())` (best-effort).
pub fn check_immutable(path: &Path) -> Result<(), String> {
    // Heuristic via lsattr -d; best-effort and non-fatal when unavailable
    let output = match std::process::Command::new("lsattr")
        .arg("-d")
        .arg(path) // avoid lossy UTF-8 conversion
        .output()
    {
        Ok(o) => o,
        Err(_) => return Ok(()), // tool not found or couldn't run -> best-effort: assume not immutable
    };

    if !output.status.success() {
        return Ok(()); // non-zero exit from lsattr -> treat as inconclusive
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(attrs) = line.split_whitespace().next() {
            if attrs.contains('i') {
                return Err(format!(
                    "Target '{}' is immutable (chattr +i). Run: chattr -i -- {}",
                    path.display(),
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

/// Source trust checks. Returns Err(String) if untrusted and `force` is false. When `force` is true,
/// returns Ok(()) and leaves it to callers to emit warnings.
pub fn check_source_trust(source: &Path, force: bool) -> Result<(), String> {
    let meta = fs::symlink_metadata(source).map_err(|e| format!("{}", e))?;
    let mode = meta.mode();
    if (mode & 0o002) != 0 && !force {
        return Err(format!(
            "Untrusted source (world-writable): {}. Pass --force to override.",
            source.display()
        ));
    }
    if meta.uid() != 0 && !force {
        return Err(format!(
            "Untrusted source (not root-owned): {}. Pass --force to override.",
            source.display()
        ));
    }
    ensure_mount_rw_exec(source)?;
    if let Ok(home) = std::env::var("HOME") {
        let home_p = Path::new(&home);
        if source.starts_with(home_p) && !force {
            return Err(format!(
                "Untrusted source under HOME: {}. Pass --force to override.",
                source.display()
            ));
        }
    }
    Ok(())
}
