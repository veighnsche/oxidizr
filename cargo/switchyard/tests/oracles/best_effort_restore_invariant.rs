//! Best-effort restore invariant assertion
//! Assert tolerate missing payload when enabled.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn best_effort_restore_invariant() {
    // Best-effort restore invariant (P2)
    // Assert tolerate missing payload when enabled
    
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.best_effort_restore = true; // Enable best-effort restore
    policy.apply.override_preflight = true; // Skip preflight checks for this test
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager
    
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Create a restore target without a backup
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&tgt, b"current").unwrap();
    
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    
    let input = PlanInput { 
        link: vec![], 
        restore: vec![RestoreRequest { target: t }] 
    };
    
    let plan = api.plan(input);
    
    // Apply with best-effort restore should succeed even without backup
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();
    
    // In best-effort mode, missing backups should not cause E_BACKUP_MISSING errors
    assert!(apply_result.errors.is_empty(), "best-effort restore should not report backup missing errors");
}
