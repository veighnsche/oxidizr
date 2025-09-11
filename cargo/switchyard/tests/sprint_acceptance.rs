use switchyard; // crate name per Cargo.toml
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
fn golden_minimal_plan_preflight_apply() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.allow_degraded_fs = true;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Setup temp tree
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/ls"), b"old").unwrap();

    // Build plan
    let src = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let tgt = SafePath::from_rooted(root, &root.join("usr/bin/ls")).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: src, target: tgt }], restore: vec![] });

    // Preflight + Apply(DryRun)
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Normalize events and compare to a minimal golden structure
    let mut got: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| {
            let mut v = redact_event(f.clone());
            if let Some(o) = v.as_object_mut() {
                o.remove("path");
            }
            v
        })
        .collect();
    got.sort_by(|a, b| {
        let ka = format!(
            "{}:{}",
            a.get("stage").and_then(|v| v.as_str()).unwrap_or(""),
            a.get("action_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        let kb = format!(
            "{}:{}",
            b.get("stage").and_then(|v| v.as_str()).unwrap_or(""),
            b.get("action_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        ka.cmp(&kb)
    });

    // The golden just asserts presence of key stages with schema_version and plan_id
    assert!(got.iter().any(|e| e.get("stage") == Some(&Value::from("plan"))));
    assert!(got.iter().any(|e| e.get("stage") == Some(&Value::from("preflight"))));
    assert!(got.iter().any(|e| e.get("stage") == Some(&Value::from("apply.attempt"))));
    assert!(got.iter().any(|e| e.get("stage") == Some(&Value::from("apply.result"))));
    for e in &got {
        assert_eq!(e.get("schema_version"), Some(&Value::from(1)));
        assert!(e.get("plan_id").is_some());
        assert!(e.get("decision").is_some());
    }
}
