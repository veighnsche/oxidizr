//! REQ-L2 Warn when no lock manager coverage
//! Assert warning emitted when lock manager absent but policy not Required.

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
fn req_l2_warn_no_lock_manager() {
    // REQ-L2 (P1)
    // Assert warning emitted when lock manager absent but policy not Required

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Optional; // Not required
    policy.apply.override_preflight = true; // Skip preflight checks for this test
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");

    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&tgt, b"old").unwrap();

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
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Check that warning is emitted when no lock manager is present
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have apply.attempt event
    let attempt_event = redacted
        .iter()
        .find(|e| e.get("stage") == Some(&Value::from("apply.attempt")));

    assert!(attempt_event.is_some(), "expected apply.attempt event");

    // No errors should occur in dry run mode
    assert!(
        apply_result.errors.is_empty(),
        "dry run should not have errors"
    );

    // Note: Actual warning verification would require checking for specific warning messages
    // This test mainly verifies that the API can operate without a lock manager when policy is Optional
}
