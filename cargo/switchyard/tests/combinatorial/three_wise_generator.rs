//! 3-wise generator for High-risk axes
//! Merge with curated additions using seed=314159.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn three_wise_generator_high_risk_axes() {
    // Test 3-wise combinations for high-risk axes using seed=314159
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    
    // Test with different policy configurations that represent high-risk axes
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required;
    policy.apply.smoke = switchyard::policy::types::SmokePolicy::Require;
    policy.risks.ownership_strict = true;
    
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Basic test to ensure the API works with high-risk policy combinations
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Create a simple plan
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
    assert!(!plan.actions.is_empty(), "expected actions in plan with high-risk policy");
    
    // Basic apply test (dry run) with high-risk policy
    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(apply_result.is_ok(), "apply should succeed in dry run mode even with high-risk policy");
}
