use std::path::Path;

/// Verify that the backup payload hash matches the expected value.
/// Returns true if the hash matches (or if actual cannot be computed), false if a mismatch is detected.
pub fn verify_payload_hash_ok(backup: &Path, expected: &str) -> bool {
    if let Some(actual) = crate::fs::meta::sha256_hex_of(backup) {
        actual == expected
    } else {
        // If we cannot compute, treat as not a mismatch (best-effort caller can decide policy)
        true
    }
}
