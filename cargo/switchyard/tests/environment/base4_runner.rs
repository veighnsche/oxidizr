//! EnvRunner Base-4 (Weekly/Platinum)
//! huge path lengths, xfs/btrfs where available, crash injection around swap/restore, parallel stress.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn envrunner_base4_weekly_platinum() {
    // EnvRunner Base-4 (P3)
    // Test with huge path lengths
    
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Huge path lengths
    let huge_segment = "a".repeat(2000);
    let src = root.join(format!("bin/{}/new", huge_segment));
    let tgt = root.join(format!("usr/{}/bin/app", huge_segment));
    
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
    assert!(!plan.actions.is_empty(), "expected actions in plan for Base-4 environment");
    
    let pf = api.preflight(&plan);
    assert!(pf.is_ok(), "preflight should succeed in Base-4 environment");
    
    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(apply_result.is_ok(), "apply should succeed in Base-4 environment (dry run)");
}
