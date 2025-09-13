use std::path::Path;

/// Verify that the backup payload hash matches the expected value.
/// Returns true if the hash matches (or if actual cannot be computed), false if a mismatch is detected.
#[must_use]
pub fn verify_payload_hash_ok(backup: &Path, expected: &str) -> bool {
    if let Some(actual) = crate::fs::meta::sha256_hex_of(backup) {
        actual == expected
    } else {
        // If we cannot compute, treat as not a mismatch (best-effort caller can decide policy)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrity_ok_when_hash_matches() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("payload");
        std::fs::write(&f, b"abc").unwrap();
        let h = crate::fs::meta::sha256_hex_of(&f).unwrap();
        assert!(verify_payload_hash_ok(&f, &h));
    }

    #[test]
    fn integrity_mismatch_detected() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("payload");
        std::fs::write(&f, b"abc").unwrap();
        let wrong = "deadbeef".to_string();
        assert!(!verify_payload_hash_ok(&f, &wrong));
    }
}
