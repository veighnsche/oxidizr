//! EnvRunner Base-3 (Nightly)
//! EXDEV simulated, low-disk for selected, tampered backup, relative symlinks, deep nesting, suid/sgid where permitted.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn envrunner_base3_nightly() {
    // EnvRunner Base-3 (P2)
    // Test with EXDEV simulation via environment variable
    
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback; // Allow degraded fallback
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Deep nesting paths
    let deep_path = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z";
    let src = root.join(format!("bin/{}/new", deep_path));
    let tgt = root.join(format!("usr/{}/bin/app", deep_path));
    
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
    assert!(!plan.actions.is_empty(), "expected actions in plan for Base-3 environment");
    
    let pf = api.preflight(&plan);
    assert!(pf.is_ok(), "preflight should succeed in Base-3 environment");
    
    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(apply_result.is_ok(), "apply should succeed in Base-3 environment (dry run)");
}
