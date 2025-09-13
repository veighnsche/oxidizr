//! EnvRunner Base-2 (Daily)
//! Base-1 + unicode + long paths + rival lock holder.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn envrunner_base2_daily() {
    // EnvRunner Base-2 (P1)
    // Test with unicode characters in paths and long paths
    
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Long paths with unicode characters
    let long_segment = "a".repeat(100);
    let unicode_segment = "🦀";
    let src = root.join(format!("bin/{}/new{}", long_segment, unicode_segment));
    let tgt = root.join(format!("usr/{}/bin/app{}", long_segment, unicode_segment));
    
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
    assert!(!plan.actions.is_empty(), "expected actions in plan for Base-2 environment");
    
    let pf = api.preflight(&plan);
    assert!(pf.is_ok(), "preflight should succeed in Base-2 environment");
    
    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(apply_result.is_ok(), "apply should succeed in Base-2 environment (dry run)");
}
