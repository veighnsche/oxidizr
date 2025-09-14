//! Smoke invariants assertion
//! Assert missing/fail leads to E_SMOKE and auto-rollback per policy.

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

// Mock smoke runner that always fails
#[derive(Debug)]
struct FailingSmokeRunner;
impl switchyard::adapters::SmokeTestRunner for FailingSmokeRunner {
    fn run(
        &self,
        _plan: &switchyard::types::plan::Plan,
    ) -> Result<(), switchyard::adapters::SmokeFailure> {
        Err(switchyard::adapters::SmokeFailure)
    }
}

#[test]
fn smoke_invariants() {
    // Smoke invariants (P2)
    // Assert missing/fail leads to E_SMOKE and auto-rollback per policy

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.smoke = switchyard::policy::types::SmokePolicy::Require {
        auto_rollback: true,
    }; // Require smoke tests
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_smoke_runner(Box::new(FailingSmokeRunner));

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

    // Apply with failing smoke runner: under Option A (facts-only), apply returns Ok(report)
    // and records E_SMOKE in facts and report.errors; auto-rollback may occur per policy.
    let report = api.apply(&plan, ApplyMode::Commit).unwrap();
    assert!(
        !report.errors.is_empty(),
        "apply.report should contain errors when smoke test fails"
    );
    assert!(
        report.rolled_back,
        "policy requires auto_rollback=true, report should indicate rolled_back"
    );

    // Check that we got the appropriate smoke error events
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have an apply.attempt or apply.result failure with E_SMOKE error
    assert!(
        redacted
            .iter()
            .any(|e| { e.get("error_id") == Some(&Value::from("E_SMOKE")) }),
        "expected E_SMOKE error in events"
    );
}
