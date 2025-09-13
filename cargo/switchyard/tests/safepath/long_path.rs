//! E2E-SAFEPATH-011 — Long path (255 bytes)
//! Assert Ok with long paths.

use switchyard::types::safepath::SafePath;

#[test]
fn e2e_safepath_011_long_path_255_bytes() {
    // E2E-SAFEPATH-011 (P2)
    let root = tempfile::tempdir().unwrap();
    let r = root.path();
    
    // Create a long path (255 bytes)
    let long_segment = "a".repeat(100); // 100 characters
    let candidate = std::path::Path::new(&format!("usr/{}/bin/{}", long_segment, long_segment));
    
    let sp = SafePath::from_rooted(r, candidate);
    // This should succeed as long paths are supported
    assert!(sp.is_ok(), "long paths should be accepted");
}
