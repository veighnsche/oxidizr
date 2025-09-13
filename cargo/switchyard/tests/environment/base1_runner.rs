//! EnvRunner Base-1 (CI quick)
//! ext4-like tmpfs, same-fs, normal disk, single-thread; unicode off; short paths; file lock manager present.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn envrunner_base1_ci_quick() {
    // EnvRunner Base-1 (P0)
    // Test with normal conditions: same filesystem, single thread, short paths, unicode off

    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts, audit, policy);

    // Use temp directory (simulates tmpfs/same-fs environment)
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    // Short paths
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
    assert!(
        !plan.actions.is_empty(),
        "expected actions in plan for Base-1 environment"
    );

    let pf = api.preflight(&plan);
    assert!(pf.is_ok(), "preflight should succeed in Base-1 environment");

    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(
        apply_result.is_ok(),
        "apply should succeed in Base-1 environment (dry run)"
    );
}
