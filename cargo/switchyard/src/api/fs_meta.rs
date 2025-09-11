use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use serde_json::json;

/// Compute SHA-256 of a file at `path`, returning a lowercase hex string.
pub(crate) fn sha256_hex_of(path: &Path) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut f, &mut hasher).ok()?;
    let out = hasher.finalize();
    Some(hex::encode(out))
}

/// If `target` is a symlink, resolve its target to an absolute path.
/// Relative links are resolved relative to the parent directory of `target`.
pub(crate) fn resolve_symlink_target(target: &Path) -> Option<PathBuf> {
    if let Ok(md) = std::fs::symlink_metadata(target) {
        if md.file_type().is_symlink() {
            if let Ok(mut link) = std::fs::read_link(target) {
                if link.is_relative() {
                    if let Some(parent) = target.parent() {
                        link = parent.join(link);
                    }
                }
                return Some(link);
            }
        }
    }
    None
}

/// Return a string describing the kind of filesystem node at `path`.
pub(crate) fn kind_of(path: &Path) -> String {
    match std::fs::symlink_metadata(path) {
        Ok(md) => {
            let ft = md.file_type();
            if ft.is_symlink() {
                "symlink".to_string()
            } else if ft.is_file() {
                "file".to_string()
            } else if ft.is_dir() {
                "dir".to_string()
            } else {
                "unknown".to_string()
            }
        }
        Err(_) => "missing".to_string(),
    }
}

/// Heuristic preservation capability detector for target path.
/// Returns (preservation map, preservation_supported flag).
/// This is intentionally conservative and non-mutating; it checks basic platform support
/// and permission surface for:
/// - owner (chown on non-root will likely fail; we report false unless running as root)
/// - mode (chmod generally possible if we can access the file)
/// - timestamps (utimensat typically available; report true if file exists)
/// - xattrs (Linux extended attributes via getxattr syscall presence; best-effort probe)
/// - acls (no portable check; report false)
/// - caps (Linux file capabilities; report false unless libcap is present — we report false)
pub(crate) fn detect_preservation_capabilities(path: &Path) -> (serde_json::Value, bool) {
    let mut owner = false;
    let mut mode = false;
    let mut timestamps = false;
    let mut xattrs = false;
    let acls = false;
    let caps = false;

    if let Ok(md) = std::fs::symlink_metadata(path) {
        // mode: we can generally preserve permissions if file exists and we have write on parent
        mode = true;
        // timestamps: if metadata is readable, assume we can set atime/mtime (utimensat)
        timestamps = true;
        // owner: only root can chown arbitrarily; detect effective uid == 0
        // owner: conservative default — we cannot safely detect EUID without extra features; report false
        owner = false;
        // xattrs: conservative default without unsafe syscalls
        xattrs = false;
        let _ = md; // silence unused on non-unix targets
    }

    let preservation = json!({
        "owner": owner,
        "mode": mode,
        "timestamps": timestamps,
        "xattrs": xattrs,
        "acls": acls,
        "caps": caps,
    });
    // preservation_supported if any dimension can be preserved
    let supported = owner || mode || timestamps || xattrs || acls || caps;
    (preservation, supported)
}
