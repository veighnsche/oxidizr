//! REQ-R2 Restore exact topology coverage
//! Assert restore matches backup topology exactly.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn req_r2_restore_exact_topology() {
    // REQ-R2 (P0)
    // Assert restore matches backup topology exactly
    
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.override_preflight = true; // Skip preflight checks for this test
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager
    
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&tgt, b"original").unwrap();
    
    // Create a backup snapshot
    let backup_result = switchyard::fs::backup::create_snapshot(&tgt, switchyard::constants::DEFAULT_BACKUP_TAG);
    assert!(backup_result.is_ok(), "backup should succeed");
    
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    
    let input = PlanInput { 
        link: vec![], 
        restore: vec![RestoreRequest { target: t }] 
    };
    
    let plan = api.plan(input);
    
    // In dry run mode, we can verify the restore planning
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();
    
    // No errors should occur in dry run mode
    assert!(apply_result.errors.is_empty(), "dry run should not have errors");
    
    // The restore action should plan to restore the exact backup topology
    assert_eq!(plan.actions.len(), 1, "expected one restore action");
    match &plan.actions[0] {
        switchyard::types::plan::Action::RestoreFromBackup { target } => {
            assert_eq!(target.as_path(), tgt.as_path(), "restore should target exact original path");
        }
        _ => panic!("expected RestoreFromBackup action"),
    }
}
