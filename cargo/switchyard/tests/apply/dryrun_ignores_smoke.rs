//! E2E-APPLY-018 — DryRun ignores smoke; success, no E_SMOKE (REQ-H3 bound to Commit only)

use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[derive(Default, Clone)]
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
fn dryrun_does_not_invoke_smoke_runner() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.smoke = switchyard::policy::types::SmokePolicy::Require { auto_rollback: true };
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

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
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    let redacted: Vec<Value> = facts.events.lock().unwrap().iter().map(|(_, _, _, f)| redact_event(f.clone())).collect();
    // Ensure there is no E_SMOKE reported
    assert!(
        !redacted.iter().any(|e| e.get("error_id") == Some(&Value::from("E_SMOKE"))),
        "DryRun should not emit E_SMOKE even when smoke is required"
    );
}
