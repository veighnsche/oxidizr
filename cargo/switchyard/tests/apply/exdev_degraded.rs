//! E2E-APPLY-005 — EXDEV degraded fallback used (REQ-F2)
//! Asserts per-action apply.result has degraded=true and after_kind=symlink when policy allows degraded fallback.

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
fn exdev_degraded_fallback_sets_degraded_true() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;
    policy.governance.allow_unlocked_commit = true; // allow Commit path in test

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });

    // Force EXDEV path
    std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1");
    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();
    std::env::remove_var("SWITCHYARD_FORCE_EXDEV");

    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    assert!(redacted.iter().any(|e| e.get("stage") == Some(&Value::from("apply.result"))
        && e.get("decision") == Some(&Value::from("success"))
        && e.get("degraded") == Some(&Value::from(true))
        && e.get("after_kind") == Some(&Value::from("symlink"))),
        "expected degraded=true and after_kind=symlink in successful apply.result under EXDEV fallback");
}
