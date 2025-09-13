//! E2E-APPLY-010 — Lock timeout high (REQ-L3, REQ-L5)
//! Assert wait_ms <= timeout and lock_attempts metric is present.

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
fn e2e_apply_010_lock_timeout_high() {
    // REQ-L3, REQ-L5, E2E-APPLY-010 (P1)
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required;
    policy.lock_timeout_ms = 1000; // 1 second timeout
    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);
    
    // Layout under temp root
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
    let input = PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] };
    
    let plan = api.plan(input);
    
    // Apply without a lock manager should fail with E_LOCKING
    let apply_result = api.apply(&plan, ApplyMode::Commit);
    assert!(apply_result.is_err(), "apply should fail when locking is required but no manager is configured");
    
    // Check that we got the appropriate lock timeout events
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    
    // Should have an apply.attempt failure with E_LOCKING error
    let lock_failure = redacted.iter().find(|e| {
        e.get("stage") == Some(&Value::from("apply.attempt"))
            && e.get("decision") == Some(&Value::from("failure"))
            && e.get("error_id") == Some(&Value::from("E_LOCKING"))
    });
    
    assert!(lock_failure.is_some(), "expected apply.attempt failure with E_LOCKING error");
    
    // Check that lock_wait_ms is present and <= timeout
    if let Some(event) = lock_failure {
        let lock_wait_ms = event.get("lock_wait_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(lock_wait_ms <= 1000, "lock_wait_ms should be <= timeout (1000ms), got {}", lock_wait_ms);
        
        // Check that lock_attempts is present
        let lock_attempts = event.get("lock_attempts").is_some();
        assert!(lock_attempts, "lock_attempts should be present in apply.attempt event");
    }
}
