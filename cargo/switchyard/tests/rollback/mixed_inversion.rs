//! E2E-ROLLBACK-003 — Mixed executed actions inversion (REQ-R1)

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn mixed_actions_inverse_in_reverse_order() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.allow_unlocked_commit = true;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let api = switchyard::Switchyard::new(facts, audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    // Prepare: a file at restore target and a new source for a symlink elsewhere
    let restore_t = root.join("usr/bin/restore_t");
    std::fs::create_dir_all(restore_t.parent().unwrap()).unwrap();
    std::fs::write(&restore_t, b"old").unwrap();
    let src_new = root.join("bin/new");
    let link_t = root.join("usr/bin/app");
    std::fs::create_dir_all(src_new.parent().unwrap()).unwrap();
    std::fs::create_dir_all(link_t.parent().unwrap()).unwrap();
    std::fs::write(&src_new, b"n").unwrap();
    std::fs::write(&link_t, b"o").unwrap();

    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: SafePath::from_rooted(root, &src_new).unwrap(),
            target: SafePath::from_rooted(root, &link_t).unwrap(),
        }],
        restore: vec![RestoreRequest {
            target: SafePath::from_rooted(root, &restore_t).unwrap(),
        }],
    });

    let report = api.apply(&plan, ApplyMode::Commit).unwrap();
    // Inversion should contain two restores, in reverse order of executed actions
    let inv = api.plan_rollback_of(&report);
    assert_eq!(
        inv.actions.len(),
        2,
        "inverse of mixed executed should have two restores"
    );
    // First inverse should be for the last executed (restore target)
    match &inv.actions[0] {
        switchyard::types::plan::Action::RestoreFromBackup { target } => {
            assert!(target.as_path().ends_with("usr/bin/restore_t"));
        }
        _ => panic!("expected RestoreFromBackup as first inverse action"),
    }
}
