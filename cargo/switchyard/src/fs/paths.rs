//! Path utilities for Switchyard filesystem operations.

use std::path::Path;

/// Validate path to prevent directory traversal attacks.
/// This is a conservative check used before performing mutations.
#[must_use]
pub fn is_safe_path(path: &Path) -> bool {
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return false;
        }
    }
    if let Some(path_str) = path.to_str() {
        if path_str.contains("/../") || path_str.contains("..\\") {
            return false;
        }
    }
    true
}
