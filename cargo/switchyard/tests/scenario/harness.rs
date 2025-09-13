//! Scenario harness implementation
//! Constructs temp roots, builds SafePath, applies policy knobs, runs plan/preflight/apply.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn scenario_harness_basic() {
    // Implement a basic scenario harness that:
    // 1. Constructs temp roots
    // 2. Builds SafePath
    // 3. Applies policy knobs
    // 4. Runs plan/preflight/apply
    // 5. Records redacted facts and FS state
    
    // Step 1: Construct temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Step 2: Build SafePath instances
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&tgt, b"old").unwrap();
    
    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    
    // Step 3: Apply policy knobs
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.override_preflight = true; // Skip preflight checks for this test
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager
    
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Step 4: Run plan/preflight/apply
    let input = PlanInput { 
        link: vec![LinkRequest { source: s, target: t }], 
        restore: vec![] 
    };
    
    let plan = api.plan(input);
    assert!(!plan.actions.is_empty(), "expected actions in plan");
    
    let pf = api.preflight(&plan);
    assert!(pf.is_ok(), "preflight should succeed");
    
    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(apply_result.is_ok(), "apply should succeed in dry run mode");
    
    // Step 5: Record redacted facts and FS state
    // The JsonlSink automatically records facts
    // For FS state, we can check that our files still exist
    assert!(src.exists(), "source file should still exist after dry run");
    assert!(tgt.exists(), "target file should still exist after dry run");
}
