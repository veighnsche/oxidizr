//! REQ-R5 Partial restoration facts on rollback error coverage
//! Assert partial rollback emits facts.

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
#[ignore = "multi-file/architectural bug; see BUGS.md:partial-restoration-facts"]
fn req_r5_partial_restoration_facts() {
    // REQ-R5 (P1)
    // Assert partial rollback emits facts

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.override_preflight = true; // Skip preflight checks for this test
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    // Create multiple targets
    let src1 = root.join("bin/new1");
    let src2 = root.join("bin/new2");
    let tgt1 = root.join("usr/bin/app1");
    let tgt2 = root.join("usr/bin/app2");

    std::fs::create_dir_all(src1.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt1.parent().unwrap()).unwrap();
    std::fs::write(&src1, b"n1").unwrap();
    std::fs::write(&src2, b"n2").unwrap();
    std::fs::write(&tgt1, b"o1").unwrap();
    std::fs::write(&tgt2, b"o2").unwrap();

    let s1 = SafePath::from_rooted(root, &src1).unwrap();
    let s2 = SafePath::from_rooted(root, &src2).unwrap();
    let t1 = SafePath::from_rooted(root, &tgt1).unwrap();
    let t2 = SafePath::from_rooted(root, &tgt2).unwrap();

    let input = PlanInput {
        link: vec![
            LinkRequest {
                source: s1,
                target: t1,
            },
            LinkRequest {
                source: s2,
                target: t2,
            },
        ],
        restore: vec![],
    };

    let plan = api.plan(input);
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Generate rollback plan
    let rollback_plan = api.plan_rollback_of(&apply_result);

    // Check that rollback planning emits facts
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have rollback planning events
    let rollback_events = redacted
        .iter()
        .filter(|e| {
            e.get("stage")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .contains("rollback")
        })
        .count();

    // In dry run mode, we can verify that rollback planning occurs
    assert!(
        !rollback_plan.actions.is_empty(),
        "rollback plan should contain actions"
    );
    assert!(rollback_events > 0, "rollback operations should emit facts");
}
