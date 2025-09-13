use serde_json::Value;
use switchyard::adapters::FileLockManager;
use switchyard::logging::FactsEmitter;
use switchyard::policy::Policy;
use switchyard::types::plan::LinkRequest;
use switchyard::types::plan::PlanInput;
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
fn apply_summary_includes_lock_wait_ms() {
    let facts = TestEmitter::default();
    let audit = switchyard::logging::JsonlSink::default();
    let mut policy = Policy::default();
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let lock_path = root.join("switchyard.lock");

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_lock_manager(Box::new(FileLockManager::new(lock_path)));

    // Layout
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    // Build SafePaths
    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });

    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let evs = facts.events.lock().unwrap();
    // Find apply.result and ensure lock_wait_ms is present (may be small but should exist)
    let mut found = false;
    for (_subsystem, _event, _decision, fields) in evs.iter() {
        if fields.get("stage").and_then(|v| v.as_str()) == Some("apply.result") {
            if fields.get("lock_wait_ms").is_some() {
                found = true;
                break;
            }
        }
    }
    assert!(found, "apply.result should include lock_wait_ms field");
}
