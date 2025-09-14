// compile-fail: attempting to import internal atoms from the crate root should fail
// The crate does not re-export `fs::atomic` at the root.
use switchyard::atomic::atomic_symlink_swap;

fn main() {
    let _ = atomic_symlink_swap as fn(_, _, _, _) -> _;
}
