use crate::types::plan::ApplyMode;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const TS_ZERO: &str = "1970-01-01T00:00:00Z";

pub fn now_iso() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_else(|_| TS_ZERO.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redact_masks_and_removes_expected_fields() {
        let input = json!({
            "ts": "2025-01-01T12:00:00Z",
            "duration_ms": 123,
            "lock_wait_ms": 45,
            "severity": "warn",
            "degraded": true,
            "before_hash": "abc",
            "after_hash": "def",
            "hash_alg": "sha256",
            "provenance": {"helper":"paru", "uid": 0, "gid": 0, "pkg": "coreutils"},
            "attestation": {"signature":"sig","bundle_hash":"bh","public_key_id":"pk"}
        });
        let out = redact_event(input);
        assert_eq!(out.get("ts").and_then(|v| v.as_str()), Some(TS_ZERO));
        assert!(out.get("duration_ms").is_none());
        assert!(out.get("lock_wait_ms").is_none());
        assert!(out.get("severity").is_none());
        assert!(out.get("degraded").is_none());
        assert!(out.get("before_hash").is_none());
        assert!(out.get("after_hash").is_none());
        assert!(out.get("hash_alg").is_none());
        let prov = out.get("provenance").and_then(|v| v.as_object()).unwrap();
        assert_eq!(prov.get("helper").and_then(|v| v.as_str()), Some("***"));
        let att = out.get("attestation").and_then(|v| v.as_object()).unwrap();
        assert_eq!(att.get("signature").and_then(|v| v.as_str()), Some("***"));
        assert_eq!(att.get("bundle_hash").and_then(|v| v.as_str()), Some("***"));
        assert_eq!(att.get("public_key_id").and_then(|v| v.as_str()), Some("***"));
    }
}

/// Return a timestamp for facts emission based on mode.
/// - DryRun: constant zero timestamp for determinism.
/// - Commit: real, current timestamp in RFC3339.
pub fn ts_for_mode(mode: &ApplyMode) -> String {
    match mode {
        ApplyMode::DryRun => TS_ZERO.to_string(),
        ApplyMode::Commit => now_iso(),
    }
}

/// Apply redactions to a fact event for comparison and safe logging.
/// Currently zeroes timestamps to TS_ZERO and removes volatile fields that
/// could leak secrets in tests. Extend as policy evolves.
pub fn redact_event(mut v: Value) -> Value {
    if let Some(obj) = v.as_object_mut() {
        obj.insert("ts".into(), Value::String(TS_ZERO.to_string()));
        // Remove or normalize volatile timings
        obj.remove("duration_ms");
        obj.remove("lock_wait_ms");
        // Remove volatile flags derived from runtime conditions
        obj.remove("severity");
        obj.remove("degraded");
        // Remove content-hash fields for determinism gating (kept in raw logs)
        obj.remove("before_hash");
        obj.remove("after_hash");
        obj.remove("hash_alg");
        // Placeholder secret masking: if provenance.helper exists, replace with "***"
        if let Some(p) = obj.get_mut("provenance") {
            if let Some(pobj) = p.as_object_mut() {
                if pobj.contains_key("helper") {
                    pobj.insert("helper".into(), Value::String("***".into()));
                }
            }
        }
        // Attestations are preserved, but bundle_hash/public_key_id may vary; mask if present
        if let Some(att) = obj.get_mut("attestation") {
            if let Some(aobj) = att.as_object_mut() {
                if aobj.contains_key("bundle_hash") {
                    aobj.insert("bundle_hash".into(), Value::String("***".into()));
                }
                if aobj.contains_key("public_key_id") {
                    aobj.insert("public_key_id".into(), Value::String("***".into()));
                }
                if aobj.contains_key("signature") {
                    aobj.insert("signature".into(), Value::String("***".into()));
                }
            }
        }
    }
    v
}
