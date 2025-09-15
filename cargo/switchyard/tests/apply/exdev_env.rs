use serial_test::serial;
use serde_json::Value;
use switchyard::logging::{FactsEmitter, JsonlSink, redact_event};
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

fn mk_plan() -> (switchyard::Switchyard<TestEmitter, JsonlSink>, SafePath, SafePath, TestEmitter) {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.exdev = switchyard::policy::types::ExdevPolicy::Fail; // disallow degraded fallback
    policy.governance.allow_unlocked_commit = true;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path().to_path_buf();

    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");

    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s = SafePath::from_rooted(&root, &src).unwrap();
    let t = SafePath::from_rooted(&root, &tgt).unwrap();
    (api, s, t, facts)
}

#[test]
#[serial]
fn exdev_env_default_off_even_when_force_exdev_set() {
    // Ensure no cross-talk
    std::env::remove_var("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES");
    std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1");

    let (api, s, t, facts) = mk_plan();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });
    let res = api.apply(&plan, ApplyMode::Commit).unwrap();
    assert!(res.errors.is_empty(), "apply should succeed when env overrides are not allowed");

    // Assert no E_EXDEV in apply.result
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    assert!(
        !redacted.iter().any(|e| {
            e.get("stage") == Some(&Value::from("apply.result"))
                && e.get("decision") == Some(&Value::from("failure"))
                && e.get("error_id") == Some(&Value::from("E_EXDEV"))
        }),
        "unexpected E_EXDEV failure when overrides are disabled"
    );
}

#[test]
#[serial]
fn exdev_env_injects_when_allowed_and_force_exdev_set() {
    std::env::set_var("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES", "1");
    std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1");

    let (api, s, t, facts) = mk_plan();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });
    let _ = api.apply(&plan, ApplyMode::Commit); // ignore Result to allow failure path

    // Assert E_EXDEV in apply.result
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
                && e.get("error_id") == Some(&Value::from("E_EXDEV"))
        }),
        "expected E_EXDEV failure when overrides are enabled and force exdev is set"
    );

    // Cleanup env for other tests
    std::env::remove_var("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES");
    std::env::remove_var("SWITCHYARD_FORCE_EXDEV");
}
