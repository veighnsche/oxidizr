//! Filesystem metadata helpers used by preflight/apply for Minimal Facts and gating.
//!
//! This module provides conservative, non-mutating probes for:
//! - `kind_of(path)`: classify node kind (file/dir/symlink/missing/unknown)
//! - `resolve_symlink_target(path)`: resolve symlink target to an absolute path
//! - `detect_preservation_capabilities(path)`: detect which preservation dimensions are likely
//!   supported on the current platform and under current privileges.
//!
//! Notes:
//! - Owner preservation is reported true only when the effective UID is 0 (root).
//! - xattrs support is probed via the `xattr` crate by attempting to list attributes.
//! - Timestamps and mode are reported true when metadata is readable for the path.
//! - ACLs and capabilities are conservatively reported false.
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use serde_json::json;

/// Compute SHA-256 of a file at `path`, returning a lowercase hex string.
#[must_use]
pub fn sha256_hex_of(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;

    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).ok()?;

    Some(format!("{:x}", hasher.finalize()))
}

/// If `target` is a symlink, resolve its target to an absolute path.
/// Relative link targets are resolved relative to the parent directory of `target`.
#[must_use]
pub fn resolve_symlink_target(target: &Path) -> Option<PathBuf> {
    // Quick bail if not a symlink.
    let md = std::fs::symlink_metadata(target).ok()?;
    if !md.file_type().is_symlink() {
        return None;
    }

    // Read the raw link target (may be relative).
    let link = std::fs::read_link(target).ok()?;
    if link.is_absolute() {
        return Some(link);
    }

    // Compute an absolute base dir for resolving the relative link.
    // If `target` has no parent, use "." (current dir). Then absolutize that base.
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let base_abs = if parent.is_absolute() {
        parent.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(parent)
    };

    Some(base_abs.join(link))
}

/// Return a string describing the kind of filesystem node at `path`.
#[must_use]
pub fn kind_of(path: &Path) -> &'static str {
    match std::fs::symlink_metadata(path) {
        Ok(md) => {
            let ft = md.file_type();
            if ft.is_symlink() {
                "symlink"
            } else if ft.is_file() {
                "file"
            } else if ft.is_dir() {
                "dir"
            } else {
                "unknown"
            }
        }
        Err(_) => "missing",
    }
}

/// Heuristic preservation capability detector for target path.
/// Returns (preservation map, `preservation_supported` flag).
/// This is intentionally conservative and non-mutating; it checks basic platform support
/// and permission surface for:
/// - owner (chown on non-root will likely fail; we report false unless running as root)
/// - mode (chmod generally possible if we can access the file)
/// - timestamps (utimensat typically available; report true if file exists)
/// - xattrs (Linux extended attributes via getxattr syscall presence; best-effort probe)
/// - acls (no portable check; report false)
/// - caps (Linux file capabilities; report false unless libcap is present — we report false)
#[must_use]
pub fn detect_preservation_capabilities(path: &Path) -> (serde_json::Value, bool) {
    // Defaults: everything false.
    let mut owner = false;
    let mut mode = false;
    let mut timestamps = false;

    // xattrs varies by platform; start false and probe only where supported.
    let mut xattrs = false;

    // Not (yet) probed in this crate; conservative false.
    let acls = false;
    let caps = false;

    if std::fs::symlink_metadata(path).is_ok() {
        // If we can stat the node, assume we can preserve mode & timestamps.
        mode = true;
        timestamps = true;

        // Root can chown arbitrarily; others generally cannot.
        owner = effective_uid_is_root();

        // Best-effort xattr probe: listing succeeds ⇒ likely supported.
        #[cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "macos",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            xattrs = xattr::list(path).map(|_| true).unwrap_or(false);
        }
        // Other targets: leave xattrs = false (conservative).
    }

    let preservation = json!({
        "owner": owner,
        "mode": mode,
        "timestamps": timestamps,
        "xattrs": xattrs,
        "acls": acls,
        "caps": caps,
    });

    // Supported if any dimension is preservable.
    let supported = owner || mode || timestamps || xattrs || acls || caps;

    (preservation, supported)
}

fn effective_uid_is_root() -> bool {
    #[cfg(target_os = "linux")]
    {
        rustix::process::geteuid().as_raw() == 0
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}
