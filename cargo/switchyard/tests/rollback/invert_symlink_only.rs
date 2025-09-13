//! E2E-ROLLBACK-001 — Invert only symlink actions (REQ-R1, REQ-R3)
//! Plan with a single EnsureSymlink; after Commit, plan_rollback_of should produce a RestoreFromBackup for same target.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn invert_only_symlink_actions() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.allow_unlocked_commit = true; // allow Commit without LockManager
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let api = switchyard::Switchyard::new(facts, audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    // temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t.clone(),
        }],
        restore: vec![],
    });

    let report = api.apply(&plan, ApplyMode::Commit).unwrap();

    let inv = api.plan_rollback_of(&report);
    assert_eq!(
        inv.actions.len(),
        1,
        "inverse plan should contain one action"
    );
    match &inv.actions[0] {
        switchyard::types::plan::Action::RestoreFromBackup { target } => {
            assert_eq!(
                target.as_path(),
                t.as_path(),
                "inverse restore should target original path"
            );
        }
        _ => panic!("expected RestoreFromBackup in inverse plan"),
    }
}
