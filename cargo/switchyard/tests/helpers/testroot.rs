// tests/helpers/testroot.rs
// A small helper to provide a per-test unique root with path builders.

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TestRoot {
    td: tempfile::TempDir,
}

impl TestRoot {
    pub fn new() -> Self { Self { td: tempfile::TempDir::new().expect("tempdir") } }
    pub fn path(&self) -> &Path { self.td.path() }
    pub fn join<P: AsRef<Path>>(&self, p: P) -> PathBuf { self.path().join(p) }
    pub fn usr_bin(&self, name: &str) -> PathBuf { self.join(format!("usr/bin/{name}")) }
    pub fn bin(&self, name: &str) -> PathBuf { self.join(format!("bin/{name}")) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testroot_unique() {
        let a = TestRoot::new();
        let b = TestRoot::new();
        assert_ne!(a.path(), b.path());
        std::fs::create_dir_all(a.usr_bin("app").parent().unwrap()).unwrap();
        std::fs::write(a.usr_bin("app"), b"x").unwrap();
        assert!(a.usr_bin("app").exists());
        assert!(!b.usr_bin("app").exists());
    }
}
