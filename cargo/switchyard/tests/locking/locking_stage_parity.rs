use serde_json::Value;
use switchyard::adapters::FsOwnershipOracle;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;
use switchyard::adapters::{LockManager, LockGuard};
use switchyard::types::errors::{Error, ErrorKind, Result};

#[derive(Debug)]
#[allow(dead_code)]
struct FailingLockGuard;

impl LockGuard for FailingLockGuard {}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct FailingLockManager;

impl LockManager for FailingLockManager {
    fn acquire_process_lock(&self, _timeout_ms: u64) -> Result<Box<dyn LockGuard>> {
        Err(Error {
            kind: ErrorKind::Policy,
            msg: "E_LOCKING: timeout acquiring process lock".to_string(),
        })
    }
}

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
}

#[test]
fn apply_emits_apply_result_on_lock_failure_when_require_lock_manager() {
    // Commit with require_lock_manager=true must fail early with E_LOCKING and still emit apply.result.
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required; // precedence over allow_unlocked_commit
    policy.governance.allow_unlocked_commit = true; // even if true, require_lock_manager forces fail

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(FsOwnershipOracle::default()));

    // Minimal plan under a temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"old").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });

    let _report = api.apply(&plan, ApplyMode::Commit).expect_err("apply should fail when locking is required but lock manager fails");

    // Redacted events should include both apply.attempt and apply.result failures with E_LOCKING/30
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    assert!(
        redacted.iter().any(|e| {
            e.get("stage") == Some(&Value::from("apply.attempt"))
                && e.get("decision") == Some(&Value::from("failure"))
                && e.get("error_id") == Some(&Value::from("E_LOCKING"))
                && e.get("exit_code") == Some(&Value::from(30))
        }),
        "expected E_LOCKING failure with exit_code=30 in apply.attempt (require_lock_manager)"
    );

    assert!(
        redacted.iter().any(|e| {
            e.get("stage") == Some(&Value::from("apply.result"))
                && e.get("decision") == Some(&Value::from("failure"))
                && e.get("error_id") == Some(&Value::from("E_LOCKING"))
                && e.get("exit_code") == Some(&Value::from(30))
        }),
        "expected E_LOCKING failure with exit_code=30 in apply.result (require_lock_manager)"
    );
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
fn apply_emits_apply_result_on_lock_failure() {
    // Commit without LockManager and without allow_unlocked_commit should fail early with E_LOCKING.
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let policy = Policy::default(); // allow_unlocked_commit = false by default

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(FsOwnershipOracle::default()));

    // Minimal plan under a temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"old").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });

    let _report = api.apply(&plan, ApplyMode::Commit).expect_err("apply should fail when locking is required but lock manager fails");

    // Redacted events should include both apply.attempt and apply.result failures with E_LOCKING/30
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    assert!(
        redacted.iter().any(|e| {
            e.get("stage") == Some(&Value::from("apply.attempt"))
                && e.get("decision") == Some(&Value::from("failure"))
                && e.get("error_id") == Some(&Value::from("E_LOCKING"))
                && e.get("exit_code") == Some(&Value::from(30))
        }),
        "expected E_LOCKING failure with exit_code=30 in apply.attempt"
    );

    assert!(
        redacted.iter().any(|e| {
            e.get("stage") == Some(&Value::from("apply.result"))
                && e.get("decision") == Some(&Value::from("failure"))
                && e.get("error_id") == Some(&Value::from("E_LOCKING"))
                && e.get("exit_code") == Some(&Value::from(30))
        }),
        "expected E_LOCKING failure with exit_code=30 in apply.result"
    );
}
