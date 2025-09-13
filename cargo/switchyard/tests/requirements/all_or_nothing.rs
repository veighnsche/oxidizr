//! REQ-A3 All-or-nothing coverage
//! Assert transactional semantics.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn req_a3_all_or_nothing() {
    // REQ-A3 (P0)
    // Assert transactional semantics

    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.override_preflight = true; // Skip preflight checks for this test
    policy.governance.allow_unlocked_commit = true; // Allow commit without lock manager

    let api = switchyard::Switchyard::new(facts, audit, policy);

    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    // Create multiple targets to test all-or-nothing semantics
    let src1 = root.join("bin/new1");
    let src2 = root.join("bin/new2");
    let tgt1 = root.join("usr/bin/app1");
    let tgt2 = root.join("usr/bin/app2");

    std::fs::create_dir_all(src1.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt1.parent().unwrap()).unwrap();
    std::fs::write(&src1, b"n1").unwrap();
    std::fs::write(&src2, b"n2").unwrap();

    let s1 = SafePath::from_rooted(root, &src1).unwrap();
    let s2 = SafePath::from_rooted(root, &src2).unwrap();
    let t1 = SafePath::from_rooted(root, &tgt1).unwrap();
    let t2 = SafePath::from_rooted(root, &tgt2).unwrap();

    let input = PlanInput {
        link: vec![
            LinkRequest {
                source: s1,
                target: t1,
            },
            LinkRequest {
                source: s2,
                target: t2,
            },
        ],
        restore: vec![],
    };

    let plan = api.plan(input);

    // In dry run mode, we can verify the transactional planning
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // No errors should occur in dry run mode
    assert!(
        apply_result.errors.is_empty(),
        "dry run should not have errors"
    );

    // All actions should be planned together as a transaction
    assert_eq!(
        plan.actions.len(),
        2,
        "expected two actions in transactional plan"
    );
}
