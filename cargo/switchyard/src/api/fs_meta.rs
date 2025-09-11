use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

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
