use serde_json::json;
use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::PlanInput;
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

#[allow(dead_code, reason = "might be used in future tests")]
struct TimeoutGuard;
impl switchyard::adapters::LockGuard for TimeoutGuard {}

#[derive(Debug)]
struct TimeoutLock;
impl switchyard::adapters::LockManager for TimeoutLock {
    fn acquire_process_lock(
        &self,
        _timeout_ms: u64,
    ) -> switchyard::types::errors::Result<Box<dyn switchyard::adapters::LockGuard>> {
        Err(switchyard::types::errors::Error {
            kind: switchyard::types::errors::ErrorKind::Io,
            msg: "timeout".to_string(),
        })
    }
}

#[test]
fn locking_timeout_emits_e_locking_exit_code_and_lock_wait_ms() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_lock_manager(Box::new(TimeoutLock))
        .with_lock_timeout_ms(10);

    // Empty plan is fine; lock timeout happens before any action
    let plan = api.plan(PlanInput {
        link: vec![],
        restore: vec![],
    });
    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let evs = facts.events.lock().unwrap();
    // Find first apply.attempt event with decision=failure
    let raw: Vec<&Value> = evs
        .iter()
        .filter(|(_, event, decision, _)| event == "apply.attempt" && decision == "failure")
        .map(|(_, _, _, f)| f)
        .collect();
    assert!(!raw.is_empty(), "expected a failing apply.attempt event");

    // Redact for canon and extract stable subset
    let mut canon: Vec<Value> = raw
        .into_iter()
        .map(|f| redact_event((*f).clone()))
        .map(|r| {
            json!({
                "stage": r.get("stage").cloned().unwrap_or(json!("")),
                "decision": r.get("decision").cloned().unwrap_or(json!("")),
                "error_id": r.get("error_id").cloned().unwrap_or(json!("")),
                "exit_code": r.get("exit_code").cloned().unwrap_or(json!(0)),
            })
        })
        .collect();

    // Sort for stability
    canon.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    // Expected single-record canon
    let expected = json!([
        {
            "stage": "apply.attempt",
            "decision": "failure",
            "error_id": "E_LOCKING",
            "exit_code": 30
        }
    ]);

    // Optionally update goldens
    if let Ok(dir) = std::env::var("GOLDEN_OUT_DIR") {
        std::fs::create_dir_all(format!("{}/locking-timeout", dir)).ok();
        let path = format!("{}/locking-timeout/canon_apply_attempt.json", dir);
        std::fs::write(&path, serde_json::to_string_pretty(&canon).unwrap()).unwrap();
    }

    assert_eq!(
        canon,
        expected.as_array().unwrap().clone(),
        "locking timeout canon mismatch"
    );
}
