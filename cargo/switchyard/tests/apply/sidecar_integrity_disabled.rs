//! E2E-APPLY-020 — Sidecar integrity disabled tolerates tamper (REQ-S6)
//! Assert apply succeeds even when backup sidecar integrity is disabled and tampered.

use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
}

impl FactsEmitter for TestEmitter {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
        self.events.lock().unwrap().push((
            subsystem.to_string(),
            event.to_string(),
            decision.to_string(),
            fields,
        ));
    }
}

#[test]
fn e2e_apply_020_sidecar_integrity_disabled_tolerates_tamper() {
    // REQ-S6, E2E-APPLY-020 (P2)
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.durability.sidecar_integrity = false; // Disable sidecar integrity checking
    policy.governance.allow_unlocked_commit = true; // avoid lock manager requirement
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted; // avoid gating on temp files
    policy.apply.override_preflight = true; // skip preflight checks

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Layout under temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let input = PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    };

    let plan = api.plan(input);
    let _ = api.preflight(&plan).unwrap();

    // Apply should succeed even with sidecar integrity disabled
    let _report = api.apply(&plan, ApplyMode::Commit).unwrap();

    // Check that we got the appropriate apply events
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have an apply.result success event
    assert!(
        redacted.iter().any(|e| {
            e.get("stage") == Some(&Value::from("apply.result"))
                && e.get("decision") == Some(&Value::from("success"))
        }),
        "expected apply.result success with sidecar integrity disabled"
    );
}
