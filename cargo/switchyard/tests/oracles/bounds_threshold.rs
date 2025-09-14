//! REQ-BND1 — Bounds threshold: fsync_ms under reasonable limit

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
fn bounds_threshold_fsync_ms_under_100() {
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
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let evs = facts.events.lock().unwrap();
    let mut ok = false;
    for (_, _, _, f) in evs.iter() {
        if f.get("stage") == Some(&Value::from("apply.result")) && f.get("action_id").is_some() {
            if let Some(ms) = f.get("fsync_ms").and_then(|v| v.as_u64()) { ok = ms <= 100; break; }
        }
    }
    assert!(ok, "expected fsync_ms <= 100ms in normal environment");
}
