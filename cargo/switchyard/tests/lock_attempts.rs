use serde_json::Value;
use switchyard::adapters::lock_file::FileLockManager;
use switchyard::logging::{FactsEmitter, JsonlSink};
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
fn apply_attempt_emits_lock_attempts_greater_than_one_when_contended() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.force_untrusted_source = true;

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let lock_path = root.join("switchyard.lock");

    // Spawn a holder to contend the lock briefly
    let hold_lock_path = lock_path.clone();
    let holder = std::thread::spawn(move || {
        let mgr = FileLockManager::new(hold_lock_path);
        let g = mgr.acquire_process_lock(200).expect("holder acquires");
        // Hold for a short while to force the main apply to retry
        std::thread::sleep(std::time::Duration::from_millis(120));
        drop(g);
    });

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
        .with_lock_timeout_ms(500);

    // Minimal plan
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();

    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();
    holder.join().unwrap();

    let evs = facts.events.lock().unwrap();
    // Find the apply.attempt success event and check lock_attempts
    let mut found = false;
    for (_sub, event, decision, fields) in evs.iter() {
        if event == "apply.attempt" && decision == "success" {
            if let Some(attempts) = fields.get("lock_attempts").and_then(|v| v.as_u64()) {
                assert!(attempts >= 2, "expected lock_attempts >= 2, got {}", attempts);
                found = true;
                break;
            }
        }
    }
    assert!(found, "expected apply.attempt with lock_attempts");
}
