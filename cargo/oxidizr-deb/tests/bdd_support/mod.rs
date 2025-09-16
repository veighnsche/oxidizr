// Minimal support module for oxidizr-deb BDD.
// Extend as needed (e.g., facts/audit collectors) when oxidizr-deb exposes richer observability hooks.

pub mod util {
    use std::path::{Path, PathBuf};

    pub fn under_root<P: AsRef<Path>>(root: &Path, p: P) -> PathBuf {
        let p = p.as_ref();
        if p.is_absolute() {
            root.join(p.strip_prefix("/").unwrap())
        } else {
            root.join(p)
        }
    }
}
