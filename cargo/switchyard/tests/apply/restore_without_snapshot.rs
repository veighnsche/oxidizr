//! E2E-APPLY-016 — Restore without capture snapshot (policy.capture_restore_snapshot=false)
//! Verify that plan_rollback_of does not invert the restore action when snapshot capture is disabled.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn restore_without_capture_snapshot_does_not_invert() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.capture_restore_snapshot = false; // disable snapshot capture
    policy.governance.allow_unlocked_commit = true;

    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    // Prepare a target file and a prior backup by creating a snapshot first
    let tgt = root.join("usr/bin/app");
    std::fs::write(&tgt, b"old").unwrap();
    let _ =
        switchyard::fs::backup::create_snapshot(&tgt, switchyard::constants::DEFAULT_BACKUP_TAG);
    // Now mutate target so restore has an effect
    std::fs::write(&tgt, b"new").unwrap();

    let sp_t = SafePath::from_rooted(root, &tgt).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![],
        restore: vec![RestoreRequest {
            target: sp_t.clone(),
        }],
    });
    let report = api.apply(&plan, ApplyMode::Commit).unwrap();

    let inv = api.plan_rollback_of(&report);
    // Since capture_restore_snapshot=false, inverse should not include an entry for the restore action
    assert!(
        inv.actions.is_empty(),
        "expected no inverse actions when snapshot capture disabled"
    );
}
