use crate::config::{aur_helpers, paths};
use std::path::{Path, PathBuf};

pub(crate) fn backup_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("backup");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{}{}", name, paths::BACKUP_SUFFIX))
}

pub(crate) fn pacman_locked() -> bool {
    Path::new(paths::PACMAN_LOCK).exists()
}

pub(crate) fn aur_helper_candidates(configured: &str) -> Vec<&str> {
    if !configured.is_empty() {
        let mut helpers = vec![configured];
        helpers.extend_from_slice(&aur_helpers::DEFAULT_HELPERS);
        helpers
    } else {
        aur_helpers::DEFAULT_HELPERS.to_vec()
    }
}

// Validate package names to prevent command injection
#[allow(dead_code)]
pub(crate) fn is_valid_package_name(name: &str) -> bool {
    // Package names should only contain alphanumeric, dash, underscore, plus, and dot
    // and should not start with dash
    if name.is_empty() || name.starts_with('-') {
        return false;
    }
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '+' || c == '.')
}

// Validate paths to prevent directory traversal attacks
pub(crate) fn is_safe_path(path: &Path) -> bool {
    // Check for directory traversal patterns
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return false;
        }
    }
    // Check for absolute paths that go outside expected directories
    if let Some(path_str) = path.to_str()
        && (path_str.contains("/../") || path_str.contains("..\\"))
    {
        return false;
    }
    true
}
