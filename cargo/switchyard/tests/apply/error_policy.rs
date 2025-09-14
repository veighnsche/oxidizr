use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;
#[path = "../helpers/env.rs"]
mod scoped_env;

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
fn apply_refuses_with_e_policy_when_preflight_gates_fail_and_override_is_false() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.rescue.require = true; // require rescue availability
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted; // avoid unrelated stops
    policy.apply.override_preflight = false; // fail-closed
    policy.governance.allow_unlocked_commit = true; // allow Commit without LockManager for this test

    // Force rescue check to fail deterministically (scoped; allow env overrides in tests only)
    let _allow = scoped_env::ScopedEnv::set("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES", "1");
    let _rescue = scoped_env::ScopedEnv::set("SWITCHYARD_FORCE_RESCUE_OK", "0");

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
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

    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    // ScopedEnv drops restore env here

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
                && e.get("error_id") == Some(&Value::from("E_POLICY"))
                && e.get("exit_code") == Some(&Value::from(10))
        }),
        "expected E_POLICY failure with exit_code=10 in apply.result"
    );
}
