use std::path::{Component, Path, PathBuf};

use super::errors::{Error, ErrorKind, Result};

/// Data-only type for safe path handling.
/// Centralized under `crate::types` for cross-layer reuse.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafePath {
    /// The root path that this safe path is relative to
    root: PathBuf,
    /// The relative path component
    rel: PathBuf,
}

impl SafePath {
    /// Creates a new SafePath from a root and candidate path.
    /// 
    /// This function ensures that the candidate path is within the root path
    /// and does not contain any unsafe components like dotdot (..).
    /// 
    /// # Arguments
    /// 
    /// * `root` - The root path that the candidate should be within
    /// * `candidate` - The path to check and make safe
    /// 
    /// # Returns
    /// 
    /// * `Result<Self>` - A SafePath if the candidate is valid, or an error otherwise
    pub fn from_rooted(root: &Path, candidate: &Path) -> Result<Self> {
        if !root.is_absolute() {
            return Err(Error {
                kind: ErrorKind::InvalidPath,
                msg: "root must be absolute".into(),
            });
        }
        let effective = if candidate.is_absolute() {
            match candidate.strip_prefix(root) {
                Ok(p) => p.to_path_buf(),
                Err(_) => {
                    return Err(Error {
                        kind: ErrorKind::Policy,
                        msg: "path escapes root".into(),
                    })
                }
            }
        } else {
            candidate.to_path_buf()
        };

        let mut rel = PathBuf::new();
        for seg in effective.components() {
            match seg {
                Component::CurDir => {}
                Component::Normal(p) => rel.push(p),
                Component::ParentDir => {
                    return Err(Error {
                        kind: ErrorKind::Policy,
                        msg: "dotdot".into(),
                    });
                }
                _ => {
                    return Err(Error {
                        kind: ErrorKind::InvalidPath,
                        msg: "unsupported component".into(),
                    });
                }
            }
        }
        let norm = root.join(&rel);
        if !norm.starts_with(root) {
            return Err(Error {
                kind: ErrorKind::Policy,
                msg: "path escapes root".into(),
            });
        }
        Ok(SafePath {
            root: root.to_path_buf(),
            rel,
        })
    }

    /// Returns the full path by joining the root and relative components.
    /// 
    /// # Returns
    /// 
    /// * `PathBuf` - The complete path
    pub fn as_path(&self) -> PathBuf {
        self.root.join(&self.rel)
    }

    /// Returns a reference to the relative path component.
    /// 
    /// # Returns
    /// 
    /// * `&Path` - Reference to the relative path
    pub fn rel(&self) -> &Path {
        &self.rel
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rejects_dotdot() {
        let root = Path::new("/tmp");
        assert!(SafePath::from_rooted(root, Path::new("../etc")).is_err());
    }

    #[test]
    fn accepts_absolute_inside_root() {
        let root = Path::new("/tmp/root");
        let candidate = Path::new("/tmp/root/usr/bin/ls");
        let sp = SafePath::from_rooted(root, candidate).unwrap_or_else(|e| panic!("Failed to create SafePath for absolute path inside root: {e}"));
        assert!(sp.as_path().starts_with(root));
        assert_eq!(sp.rel(), Path::new("usr/bin/ls"));
    }

    #[test]
    fn rejects_absolute_outside_root() {
        let root = Path::new("/tmp/root");
        let candidate = Path::new("/etc/passwd");
        assert!(SafePath::from_rooted(root, candidate).is_err());
    }

    #[test]
    fn normalizes_curdir_components() {
        let root = Path::new("/tmp/root");
        let candidate = Path::new("./usr/./bin/./ls");
        let sp = SafePath::from_rooted(root, candidate).unwrap_or_else(|e| panic!("Failed to create SafePath with normalized curdir components: {e}"));
        assert_eq!(sp.rel(), Path::new("usr/bin/ls"));
        assert_eq!(sp.as_path(), Path::new("/tmp/root/usr/bin/ls"));
    }
}
