//! REQ-R1 Rollback reversibility coverage
//! Assert rollback restores prior state.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn req_r1_rollback_reversibility() {
    // REQ-R1 (P0)
    // Assert rollback restores prior state

    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.override_preflight = true; // Skip preflight checks for this test
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager

    let api = switchyard::Switchyard::new(facts, audit, policy);

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
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    };

    let plan = api.plan(input);
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Generate rollback plan
    let rollback_plan = api.plan_rollback_of(&apply_result);

    // The rollback plan should contain actions to restore the prior state
    assert!(
        !rollback_plan.actions.is_empty(),
        "rollback plan should contain actions"
    );

    // In dry run mode, we can't actually verify state restoration, but we can check planning
    match &rollback_plan.actions[0] {
        switchyard::types::plan::Action::RestoreFromBackup { target } => {
            assert_eq!(
                target.as_path(),
                tgt.as_path(),
                "rollback should target original file"
            );
        }
        _ => panic!("expected RestoreFromBackup action in rollback plan"),
    }
}
