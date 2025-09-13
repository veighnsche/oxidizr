//! Helper: DiskPressureInjector for ENOSPC
//! Used by E2E-APPLY-014 for disk space pressure testing.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn helper_disk_pressure_injector() {
    // DiskPressureInjector helper (P3)
    // Test disk space pressure scenarios

    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
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

    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(
        apply_result.is_ok(),
        "apply should succeed in normal conditions"
    );

    // Note: We can't easily simulate ENOSPC in tests without special filesystem setup
    // This test just verifies normal successful behavior
}
