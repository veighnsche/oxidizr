//! Rollback pure function assertion
//! Assert inverse plan derived from executed actions and is a pure function.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn rollback_pure_function() {
    // Rollback pure function assertion (P0)
    // Assert inverse plan derived from executed actions and is a pure function

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

    // The rollback plan should be derived from executed actions
    assert!(
        !rollback_plan.actions.is_empty(),
        "rollback plan should contain actions"
    );

    // Plan rollback should be a pure function (deterministic)
    let rollback_plan2 = api.plan_rollback_of(&apply_result);

    // Both rollback plans should be identical
    let rollback_plan_id = switchyard::types::ids::plan_id(&rollback_plan);
    let rollback_plan2_id = switchyard::types::ids::plan_id(&rollback_plan2);
    assert_eq!(
        rollback_plan_id, rollback_plan2_id,
        "rollback plan ID should be deterministic"
    );
    assert_eq!(
        rollback_plan.actions.len(),
        rollback_plan2.actions.len(),
        "rollback plans should have same number of actions"
    );

    // Verify rollback plan content
    match &rollback_plan.actions[0] {
        switchyard::types::plan::Action::RestoreFromBackup { target } => {
            assert_eq!(
                target.as_path(),
                tgt.as_path(),
                "rollback should target the original file"
            );
        }
        _ => panic!("expected RestoreFromBackup action in rollback plan"),
    }
}
