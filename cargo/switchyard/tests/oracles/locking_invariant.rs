//! Locking invariant assertion
//! Assert Required+no manager leads to early E_LOCKING and no FS mutation.

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
fn locking_invariant() {
    // Locking invariant (P0)
    // Assert Required+no manager leads to early E_LOCKING and no FS mutation
    
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required;
    
    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);
    
    // Use temp directory
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
    
    let input = PlanInput { 
        link: vec![LinkRequest { source: s, target: t }], 
        restore: vec![] 
    };
    
    let plan = api.plan(input);
    
    // Apply without lock manager should fail with E_LOCKING
    let apply_result = api.apply(&plan, ApplyMode::Commit);
    assert!(apply_result.is_err(), "apply should fail when locking is required but no manager is configured");
    
    // Check that we got the appropriate lock error events
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    
    // Should have an apply.attempt failure with E_LOCKING error
    assert!(redacted.iter().any(|e| {
        e.get("stage") == Some(&Value::from("apply.attempt"))
            && e.get("decision") == Some(&Value::from("failure"))
            && e.get("error_id") == Some(&Value::from("E_LOCKING"))
    }), "expected apply.attempt failure with E_LOCKING error");
    
    // No FS mutation should occur
    assert!(tgt.exists(), "target file should still exist (no mutation occurred)");
}
