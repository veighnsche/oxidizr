//! E2E-SAFEPATH-012 — Huge path (4096 bytes)
//! Assert Ok or documented error with huge paths.

use switchyard::types::safepath::SafePath;

#[test]
fn e2e_safepath_012_huge_path_4096_bytes() {
    // E2E-SAFEPATH-012 (P3)
    let root = tempfile::tempdir().unwrap();
    let r = root.path();
    
    // Create a huge path (4096 bytes)
    let huge_segment = "a".repeat(2000); // 2000 characters
    let path_str = format!("usr/{}/bin/{}", huge_segment, huge_segment);
    let candidate = std::path::Path::new(&path_str);
    
    let sp = SafePath::from_rooted(r, candidate);
    // This should either succeed or fail with a documented error
    // On most systems, this will succeed as modern filesystems support long paths
    assert!(sp.is_ok(), "huge paths should be accepted or fail with documented error");
}
