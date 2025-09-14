//! REQ-O5 — Before/after hashes present in apply.result per-action facts

use serde_json::Value;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
}
impl switchyard::logging::FactsEmitter for TestEmitter {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
        self.events
            .lock()
            .unwrap()
            .push((subsystem.to_string(), event.to_string(), decision.to_string(), fields));
    }
}

#[test]
fn req_o5_before_after_hashes_present() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.override_preflight = true;
    policy.governance.allow_unlocked_commit = true;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&tgt, b"old").unwrap();

    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    // Use Commit mode so fields are not redacted out of emitted facts
    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let events = facts.events.lock().unwrap();
    let mut ok = false;
    for (_, _, _, f) in events.iter() {
        if f.get("stage") == Some(&Value::from("apply.result"))
            && f.get("action_id").is_some()
            && f.get("hash_alg") == Some(&Value::from("sha256"))
            && f.get("before_hash").is_some()
            && f.get("after_hash").is_some()
        {
            ok = true;
            break;
        }
    }
    assert!(ok, "expected sha256 before/after hash fields in apply.result per-action fact");
}
