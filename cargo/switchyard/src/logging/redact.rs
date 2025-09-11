use crate::types::plan::ApplyMode;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const TS_ZERO: &str = "1970-01-01T00:00:00Z";

pub fn now_iso() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_else(|_| TS_ZERO.to_string())
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
