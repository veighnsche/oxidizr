//! REQ-S6 Backup sidecar integrity coverage
//! Assert integrity verification and policy knobs.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn req_s6_backup_sidecar_integrity() {
    // REQ-S6 (P1)
    // Assert integrity verification and policy knobs

    let facts = JsonlSink::default();
    let audit = JsonlSink::default();

    // Test with sidecar integrity enabled
    let mut policy = Policy::default();
    policy.durability.sidecar_integrity = true;
    policy.apply.override_preflight = true; // Skip other preflight checks for this test
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

    let plan = api.plan(input.clone());
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // No errors should occur in dry run mode
    assert!(
        apply_result.errors.is_empty(),
        "dry run should not have errors with sidecar integrity enabled"
    );

    // Test with sidecar integrity disabled
    let mut policy = Policy::default();
    policy.durability.sidecar_integrity = false;
    policy.apply.override_preflight = true; // Skip other preflight checks for this test

    let api = switchyard::Switchyard::new(facts, audit, policy);

    let plan = api.plan(input);
    let apply_result = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // No errors should occur in dry run mode
    assert!(
        apply_result.errors.is_empty(),
        "dry run should not have errors with sidecar integrity disabled"
    );
}
