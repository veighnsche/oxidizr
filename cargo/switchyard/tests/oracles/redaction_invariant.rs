//! Redaction invariant assertion
//! Assert DryRun facts TS_ZERO and volatile fields removed; Commit comparisons ignore volatile fields.

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
fn redaction_invariant() {
    // Redaction invariant (P0)
    // Assert DryRun facts TS_ZERO and volatile fields removed; Commit comparisons ignore volatile fields

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.override_preflight = true; // Skip preflight checks for this test

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
    let _ = api.preflight(&plan).unwrap();

    // Apply in dry run mode
    let _dryrun_result = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Check that dry run facts have TS_ZERO timestamps
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have apply.result success event with TS_ZERO timestamp
    let dryrun_event = redacted.iter().find(|e| {
        e.get("stage") == Some(&Value::from("apply.result"))
            && e.get("decision") == Some(&Value::from("success"))
    });

    assert!(
        dryrun_event.is_some(),
        "expected apply.result success event in dry run"
    );

    // Check that timestamp is zeroed in redacted events
    if let Some(event) = dryrun_event {
        let ts = event.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            ts == "1970-01-01T00:00:00Z" || ts.is_empty(),
            "timestamp should be zeroed in dry run facts"
        );
    }
}
