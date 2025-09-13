//! E2E-APPLY-010 — Lock timeout high; assert wait_ms <= timeout (REQ-L3, REQ-L5)

use serde_json::Value;
use switchyard::adapters::FileLockManager;
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
fn lock_timeout_high_acquires_and_wait_le_timeout() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let lock_path = root.join("switchyard.lock");

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
        .with_lock_timeout_ms(10_000);

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

    let redacted: Vec<Value> = facts.events.lock().unwrap().iter().map(|(_, _, _, f)| redact_event(f.clone())).collect();
    // Find apply.result and assert lock_wait_ms <= 10000 when present
    for e in &redacted {
        if e.get("stage") == Some(&Value::from("apply.result")) {
            if let Some(ms) = e.get("lock_wait_ms").and_then(|v| v.as_u64()) {
                assert!(ms <= 10_000, "lock_wait_ms should be <= timeout; got {}", ms);
            }
        }
    }
}
