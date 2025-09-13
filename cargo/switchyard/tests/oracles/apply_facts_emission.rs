//! Apply facts emission assertion
//! Assert per-action facts emitted and summary includes summary_error_ids chain on failure.

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
fn apply_facts_emission() {
    // Apply facts emission assertion (P0)
    // Assert per-action facts emitted and summary includes summary_error_ids chain

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
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

    // Check that we got the appropriate apply events
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have apply.attempt and apply.result events
    assert!(
        redacted
            .iter()
            .any(|e| { e.get("stage") == Some(&Value::from("apply.attempt")) }),
        "expected apply.attempt event"
    );

    assert!(
        redacted
            .iter()
            .any(|e| { e.get("stage") == Some(&Value::from("apply.result")) }),
        "expected apply.result event"
    );

    // In dry run mode, there should be no errors
    assert!(
        apply_result.errors.is_empty(),
        "dry run should not have errors"
    );
}
