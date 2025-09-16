use serde_json::json;
use switchyard::logging::redact::redact_event;

fn main() {
    let sample = json!({
        "ts": "2025-01-01T00:00:00Z",
        "duration_ms": 12,
        "provenance": {"helper": "paru", "uid": 0, "gid": 0, "pkg": "coreutils"},
        "attestation": {"signature": "sig", "bundle_hash": "bh", "public_key_id": "pk"}
    });
    let redacted = redact_event(sample);
    println!("{}", redacted);
}
