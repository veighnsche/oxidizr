// This test intentionally attempts to call a mutating API with &Path instead of SafePath.
// It should fail to compile (REQ-API1: mutating APIs must accept SafePath only).

use switchyard::fs::swap::replace_file_with_symlink;

fn main() {
    let p = std::path::Path::new("foo");
    let q = std::path::Path::new("bar");
    // Wrong types: expected &SafePath for source/target
    let _ = replace_file_with_symlink(p, q, false, false, "tag");
}
