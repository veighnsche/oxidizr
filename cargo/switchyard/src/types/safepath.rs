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

    pub fn as_path(&self) -> PathBuf {
        self.root.join(&self.rel)
    }

    pub fn rel(&self) -> &Path {
        &self.rel
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

    #[test]
    fn accepts_absolute_inside_root() {
        let root = Path::new("/tmp/root");
        let candidate = Path::new("/tmp/root/usr/bin/ls");
        let sp = SafePath::from_rooted(root, candidate).expect("inside root");
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
        let sp = SafePath::from_rooted(root, candidate).expect("normalize");
        assert_eq!(sp.rel(), Path::new("usr/bin/ls"));
        assert_eq!(sp.as_path(), Path::new("/tmp/root/usr/bin/ls"));
    }
}
