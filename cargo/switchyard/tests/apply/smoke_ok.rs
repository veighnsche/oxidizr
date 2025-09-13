//! E2E-APPLY-012 — Smoke runner present and ok; assert rolled_back=false (REQ-H1)

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
fn smoke_runner_ok_yields_success_and_no_rollback() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.smoke = switchyard::policy::types::SmokePolicy::Require {
        auto_rollback: true,
    };
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;
    policy.governance.allow_unlocked_commit = true;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_smoke_runner(Box::new(switchyard::adapters::DefaultSmokeRunner::default()))
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
    let _ = api.preflight(&plan).unwrap();

    let report = api.apply(&plan, ApplyMode::Commit).unwrap();
    assert!(!report.rolled_back, "smoke ok should not trigger rollback");
    assert!(report.errors.is_empty(), "apply should be success");

    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    assert!(
        redacted
            .iter()
            .any(|e| e.get("stage") == Some(&Value::from("apply.result"))
                && e.get("decision") == Some(&Value::from("success"))),
        "expected apply.result success"
    );
}
