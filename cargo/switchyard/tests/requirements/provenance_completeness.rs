//! REQ-O7 Provenance completeness coverage
//! Assert complete provenance chain logged.

use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
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
fn req_o7_provenance_completeness() {
    // REQ-O7 (P1)
    // Assert complete provenance chain logged
    
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.override_preflight = true; // Skip preflight checks for this test
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager
    policy.risks.ownership_strict = true; // Enable ownership tracking
    
    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));
    
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
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();
    
    // Check that complete provenance chain is logged
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    
    // Should have apply.result success event with provenance information
    let apply_event = redacted.iter().find(|e| {
        e.get("stage") == Some(&Value::from("apply.result"))
            && e.get("decision") == Some(&Value::from("success"))
    });
    
    assert!(apply_event.is_some(), "expected apply.result success event with provenance");
    
    // Check that provenance includes uid/gid/pkg fields
    if let Some(event) = apply_event {
        let provenance = event.get("provenance").unwrap();
        assert!(provenance.get("uid").is_some(), "provenance should include uid field");
        assert!(provenance.get("gid").is_some(), "provenance should include gid field");
        assert!(provenance.get("pkg").is_some(), "provenance should include pkg field");
    }
    
    // No errors should occur in dry run mode
    assert!(apply_result.errors.is_empty(), "dry run should not have errors with provenance tracking");
}
