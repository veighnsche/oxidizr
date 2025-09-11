use std::path::{Component, Path, PathBuf};

use super::errors::{Error, ErrorKind, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafePath {
    root: PathBuf,
    rel: PathBuf,
}

impl SafePath {
    pub fn from_rooted(root: &Path, candidate: &Path) -> Result<Self> {
        assert!(root.is_absolute(), "root must be absolute");
        let effective = if candidate.is_absolute() {
            match candidate.strip_prefix(root) {
                Ok(p) => p.to_path_buf(),
                Err(_) => return Err(Error { kind: ErrorKind::Policy, msg: "path escapes root".into() }),
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
                    return Err(Error { kind: ErrorKind::Policy, msg: "dotdot".into() });
                }
                _ => {
                    return Err(Error { kind: ErrorKind::InvalidPath, msg: "unsupported component".into() });
                }
            }
        }
        let norm = root.join(&rel);
        if !norm.starts_with(root) {
            return Err(Error { kind: ErrorKind::Policy, msg: "path escapes root".into() });
        }
        Ok(SafePath { root: root.to_path_buf(), rel })
    }

    pub fn as_path(&self) -> PathBuf {
        self.root.join(&self.rel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rejects_dotdot() {
        let root = Path::new("/tmp");
        assert!(SafePath::from_rooted(root, Path::new("../etc")).is_err());
    }
}
