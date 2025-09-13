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
fn commit_requires_smoke_runner_when_policy_enforced() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted; // avoid preflight source trust STOP in temp env
    policy.governance.smoke = switchyard::policy::types::SmokePolicy::Require {
        auto_rollback: true,
    };
    policy.governance.allow_unlocked_commit = true; // allow Commit path in test without LockManager

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    // Setup temp tree
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/ls"), b"old").unwrap();

    // Build plan: single link action
    let src = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let tgt = SafePath::from_rooted(root, &root.join("usr/bin/ls")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: src,
            target: tgt,
        }],
        restore: vec![],
    });

    let report = api.apply(&plan, ApplyMode::Commit).unwrap();

    // Expect failure and (optional) rollback
    assert!(
        !report.errors.is_empty(),
        "apply should fail when smoke runner missing under policy"
    );

    // Redacted events should include an apply.result failure with E_SMOKE and exit_code 80
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    assert!(
        redacted.iter().any(|e| {
            e.get("stage") == Some(&Value::from("apply.result"))
                && e.get("decision") == Some(&Value::from("failure"))
                && e.get("error_id") == Some(&Value::from("E_SMOKE"))
                && e.get("exit_code") == Some(&Value::from(80))
        }),
        "expected E_SMOKE failure with exit_code=80 in apply.result"
    );
}
