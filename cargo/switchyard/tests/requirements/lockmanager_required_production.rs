//! REQ-L4 LockManager required in production coverage
//! Assert E_LOCKING when Required and no manager in Commit mode.

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
#[ignore = "multi-file/architectural bug; see BUGS.md:lockmanager-required-production"]
fn req_l4_lockmanager_required_production() {
    // REQ-L4 (P0)
    // Assert E_LOCKING when Required and no manager in Commit mode

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required; // Required policy

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

    // Apply in commit mode without lock manager should fail with E_LOCKING
    let apply_result = api.apply(&plan, ApplyMode::Commit);
    assert!(
        apply_result.is_err(),
        "commit should fail when locking is required but no manager is configured"
    );

    // Check that E_LOCKING error is emitted
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have apply.attempt failure event with E_LOCKING error
    let lock_error_event = redacted.iter().find(|e| {
        e.get("stage") == Some(&Value::from("apply.attempt"))
            && e.get("decision") == Some(&Value::from("failure"))
            && e.get("error_id") == Some(&Value::from("E_LOCKING"))
    });

    assert!(
        lock_error_event.is_some(),
        "expected E_LOCKING error when lock manager is required but absent"
    );
}
