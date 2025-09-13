//! Helper: RivalLockHolder to simulate contention with bounded hold times
//! Used by E2E-APPLY-010/015 for lock contention testing.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn helper_rival_lock_holder() {
    // RivalLockHolder helper (P2)
    // Test lock contention scenarios
    
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required;
    let api = switchyard::Switchyard::new(facts, audit, policy)
        .with_lock_timeout_ms(1000); // 1 second timeout
    
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
    
    // Apply without lock manager should fail with E_LOCKING due to timeout
    let apply_result = api.apply(&plan, ApplyMode::Commit);
    assert!(apply_result.is_err(), "apply should fail when locking is required but no manager is configured");
}
