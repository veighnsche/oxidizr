use serde_json::{json, Value};

/// Insert optional before/after hashes into a per-action extra fields object.
pub(crate) fn insert_hashes(
    extra: &mut Value,
    before_hash: Option<&String>,
    after_hash: Option<&String>,
) {
    if let Some(bh) = before_hash.as_ref() {
        if let Some(obj) = extra.as_object_mut() {
            obj.insert("hash_alg".to_string(), json!("sha256"));
            obj.insert("before_hash".to_string(), json!(bh));
        }
    }
    if let Some(ah) = after_hash.as_ref() {
        if let Some(obj) = extra.as_object_mut() {
            obj.insert("hash_alg".to_string(), json!("sha256"));
            obj.insert("after_hash".to_string(), json!(ah));
        }
    }
}

/// If fsync took longer than `warn_ms`, annotate the extra with severity=warn.
pub(crate) fn maybe_warn_fsync(extra: &mut Value, fsync_ms: u64, warn_ms: u64) {
    if fsync_ms > warn_ms {
        if let Some(obj) = extra.as_object_mut() {
            obj.insert("severity".to_string(), json!("warn"));
        }
    }
}
