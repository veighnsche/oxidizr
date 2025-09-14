use serde_json::Value;
use switchyard::adapters::{LockGuard, LockManager};
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::errors::{Error, ErrorKind, Result};
use switchyard::types::plan::PlanInput;
use switchyard::types::ApplyMode;

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, serde_json::Value)>>>,
}

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
fn commit_requires_lock_manager_when_policy_enforced() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_lock_manager(Box::new(FailingLockManager::default()));

    // Empty plan is fine; lock check happens before any action
    let plan = api.plan(PlanInput {
        link: vec![],
        restore: vec![],
    });
    let _err = api
        .apply(&plan, ApplyMode::Commit)
        .expect_err("apply should fail when locking is required but lock manager fails");

    // Find a failing apply.attempt event and assert it maps to E_LOCKING with exit_code 30
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
}
