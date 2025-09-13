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
fn apply_result_includes_perf_object() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.allow_unlocked_commit = true; // no lock manager in test

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Layout under tmp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();

    let source = SafePath::from_rooted(root, &src).unwrap();
    let target = SafePath::from_rooted(root, &tgt).unwrap();
    let input = PlanInput {
        link: vec![LinkRequest { source, target }],
        restore: vec![],
    };

    let plan = api.plan(input);
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    let perf = redacted
        .iter()
        .find_map(|e| {
            if e.get("stage") == Some(&Value::from("apply.result")) {
                e.get("perf").cloned()
            } else {
                None
            }
        })
        .expect("apply.result should include perf");

    let obj = perf.as_object().expect("perf is object");
    assert!(obj.get("hash_ms").is_some());
    assert!(obj.get("backup_ms").is_some());
    assert!(obj.get("swap_ms").is_some());
}
