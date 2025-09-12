// This test intentionally tries to import a crate-private FS atom.
// It should fail to compile.
use switchyard::fs::atomic_symlink_swap;

fn main() {
    let _ = atomic_symlink_swap;
}
