//! REQ-C1 — Dry-run by default semantics
//! Assert DryRun leaves filesystem unchanged and emits TS_ZERO timestamps.

use serde_json::Value;
use switchyard::logging::{JsonlSink, TS_ZERO};
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
fn req_c1_dry_run_by_default() {
    // We explicitly pass DryRun mode (the API requires an explicit mode parameter).
    // This test asserts the dry-run semantics: no filesystem mutation and TS_ZERO timestamps.
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    // Skip preflight gating to focus on apply surface
    policy.apply.override_preflight = true;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Under temp root, prepare a regular file target and a new source.
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
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest { source: s, target: t.clone() }],
        restore: vec![],
    });

    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Filesystem unchanged: target remains a regular file (not a symlink)
    let md = std::fs::symlink_metadata(&tgt).unwrap();
    assert!(md.file_type().is_file(), "target should remain a regular file in DryRun");

    // Facts use TS_ZERO in DryRun
    let evs = facts.events.lock().unwrap();
    assert!(
        evs.iter().any(|(_, _, _, f)| f.get("ts").and_then(|v| v.as_str()) == Some(TS_ZERO)),
        "expected TS_ZERO timestamps in DryRun facts"
    );
}
