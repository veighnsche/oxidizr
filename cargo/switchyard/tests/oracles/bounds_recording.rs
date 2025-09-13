//! Bounds recording assertion
//! Assert fsync_ms and lock_wait_ms recorded per SPEC.

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
fn bounds_recording() {
    // Bounds recording assertion (P1)
    // Assert fsync_ms and lock_wait_ms recorded per SPEC

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
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
    let _apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Check that we got the appropriate apply events
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have apply.result success event with timing metrics
    let apply_event = redacted.iter().find(|e| {
        e.get("stage") == Some(&Value::from("apply.result"))
            && e.get("decision") == Some(&Value::from("success"))
    });

    assert!(apply_event.is_some(), "expected apply.result success event");

    // Check that timing metrics are recorded
    if let Some(event) = apply_event {
        // fsync_ms should be present (even if 0 in dry run)
        assert!(
            event.get("fsync_ms").is_some(),
            "fsync_ms should be recorded"
        );

        // lock_wait_ms should be present (even if 0 when no lock manager)
        assert!(
            event.get("lock_wait_ms").is_some(),
            "lock_wait_ms should be recorded"
        );
    }
}
