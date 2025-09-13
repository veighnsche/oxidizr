//! E2E-PLAN-004 — Huge plan size (1000 links) performance and determinism (REQ-D1)
//! Assert ordering and run-time within budget.

use std::time::Instant;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn e2e_plan_004_huge_plan_performance_and_determinism() {
    // REQ-D1, E2E-PLAN-004 (P3)
    let api = switchyard::Switchyard::new(JsonlSink::default(), JsonlSink::default(), Policy::default());
    
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Build 1000 links in a deterministic order
    let mut link_reqs = Vec::new();
    for i in 0..1000 {
        let src = root.join(format!("bin/new{}", i));
        let tgt = root.join(format!("usr/bin/app{}", i));
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
        std::fs::write(&src, format!("n{}", i)).unwrap();
        std::fs::write(&tgt, format!("o{}", i)).unwrap();
        
        let sp_s = SafePath::from_rooted(root, &src).unwrap();
        let sp_t = SafePath::from_rooted(root, &tgt).unwrap();
        link_reqs.push(LinkRequest { source: sp_s, target: sp_t });
    }
    
    // Measure plan generation time
    let start = Instant::now();
    let plan = api.plan(PlanInput { link: link_reqs, restore: vec![] });
    let duration = start.elapsed();
    
    // Assert plan has 1000 actions
    assert_eq!(plan.actions.len(), 1000, "expected 1000 actions in huge plan");
    
    // Verify sorted by kind (EnsureSymlink) then by target.rel lexicographically
    let last_kind = 0u8; // 0 for link
    let mut last_t = String::new();
    for act in plan.actions.iter() {
        match act {
            switchyard::types::plan::Action::EnsureSymlink { target, .. } => {
                assert_eq!(last_kind, 0u8, "all actions should be EnsureSymlink");
                let cur = target.rel().to_string_lossy().to_string();
                assert!(cur >= last_t, "targets should be sorted: {} >= {}", cur, last_t);
                last_t = cur;
            }
            _ => panic!("expected only EnsureSymlink actions"),
        }
    }
    
    // Assert performance is within reasonable bounds (less than 5 seconds)
    assert!(duration.as_secs() < 5, "plan generation should complete within 5 seconds");
}
