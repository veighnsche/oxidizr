use serde_json::Value;
use switchyard::logging::{FactsEmitter, JsonlSink, redact_event};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

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
fn preflight_summary_failure_maps_to_e_policy_with_exit_code() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.require_rescue = true; // force a stop
    policy.force_untrusted_source = true; // avoid unrelated stops

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Minimal plan
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    // Force rescue verification failure
    std::env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "0");
    let _ = api.preflight(&plan).unwrap();
    std::env::remove_var("SWITCHYARD_FORCE_RESCUE_OK");

    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    assert!(redacted.iter().any(|e| {
        e.get("stage") == Some(&Value::from("preflight")) &&
        e.get("decision") == Some(&Value::from("failure")) &&
        e.get("error_id") == Some(&Value::from("E_POLICY")) &&
        e.get("exit_code") == Some(&Value::from(10))
    }), "expected preflight summary failure to include E_POLICY/10");
}
