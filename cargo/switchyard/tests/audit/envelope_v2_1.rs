use serde_json::Value;
use switchyard::logging::{FactsEmitter, JsonlSink, redact_event};
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
fn envelope_contains_v2_1_fields() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    // Run both preflight and apply to capture dry-run events
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    let events = facts.events.lock().unwrap().clone();
    assert!(!events.is_empty(), "expected captured events");

    // Assert envelope fields present in redacted (dry-run) output
    for (_sub, _evt, _dec, fields) in &events {
        let v = redact_event(fields.clone());
        assert_eq!(v.get("schema_version"), Some(&Value::from(2)));
        assert!(v.get("event_id").is_some(), "event_id missing");
        assert!(v.get("run_id").is_some(), "run_id missing");
        assert!(v.get("seq").is_some(), "seq missing");
        assert!(v.get("switchyard_version").is_some(), "switchyard_version missing");
        assert_eq!(v.get("dry_run"), Some(&Value::from(true)));
        assert_eq!(v.get("redacted"), Some(&Value::from(true)));
    }
}
