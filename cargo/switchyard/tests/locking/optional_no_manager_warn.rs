//! REQ-L2 — Warn when no lock manager and locking Optional

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
fn warn_emitted_when_no_lock_manager_and_optional() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    // Optional + allow_unlocked_commit true => should WARN, not fail
    policy.governance.allow_unlocked_commit = true;

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

    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let redacted: Vec<Value> = facts.events.lock().unwrap().iter().map(|(_, _, _, f)| redact_event(f.clone())).collect();
    assert!(redacted.iter().any(|e| e.get("stage") == Some(&Value::from("apply.attempt"))
        && e.get("decision") == Some(&Value::from("warn"))
        && (e.get("no_lock_manager").is_some() || e.get("lock_backend") == Some(&Value::from("none")))),
        "expected WARN apply.attempt when no lock manager and locking Optional");
}
