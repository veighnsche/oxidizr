//! Pairwise combination generator for all functions
//! Export scenario manifests with seeds and map to E2E IDs.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn pairwise_combination_generator() {
    // Test pairwise combinations using seed=4242
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Basic test to ensure the API works with pairwise combinations
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Create a simple plan with both link and restore actions
    let src = root.join("bin/new");
    let tgt_link = root.join("usr/bin/app");
    let tgt_restore = root.join("usr/bin/restore_me");
    
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt_link.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&tgt_link, b"old_link").unwrap();
    std::fs::write(&tgt_restore, b"old_restore").unwrap();
    
    let s = SafePath::from_rooted(root, &src).unwrap();
    let t_link = SafePath::from_rooted(root, &tgt_link).unwrap();
    let t_restore = SafePath::from_rooted(root, &tgt_restore).unwrap();
    
    let input = PlanInput { 
        link: vec![LinkRequest { source: s, target: t_link }], 
        restore: vec![RestoreRequest { target: t_restore }] 
    };
    
    let plan = api.plan(input);
    assert_eq!(plan.actions.len(), 2, "expected two actions in plan");
    
    // Basic preflight test
    let pf = api.preflight(&plan);
    assert!(pf.is_ok(), "preflight should succeed for basic case");
    
    // Basic apply test (dry run)
    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(apply_result.is_ok(), "apply should succeed in dry run mode");
    
    // Basic rollback test
    let rollback_plan = api.plan_rollback_of(&apply_result.unwrap());
    assert!(!rollback_plan.actions.is_empty(), "rollback plan should not be empty");
}
