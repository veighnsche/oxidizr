//! E2E-PLAN tests
//! TODO items covered:
//! - E2E-PLAN-001 (REQ-D1, REQ-O1)
//! - E2E-PLAN-003 (REQ-D1)
//! - E2E-PLAN-006 (REQ-D1)
//!
//! Traceability: requirements.yaml REQ-D1, REQ-O1; TESTPLAN/test_selection_matrix.md

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn e2e_plan_001_empty_inputs_yields_empty_actions() {
    // REQ-D1, E2E-PLAN-001 (P0)
    let api = switchyard::Switchyard::new(JsonlSink::default(), JsonlSink::default(), Policy::default());
    let plan = api.plan(PlanInput::default());
    assert!(plan.actions.is_empty(), "Plan.actions should be empty for empty input");
}

#[test]
fn e2e_plan_006_single_link_trivial_plan() {
    // REQ-D1, E2E-PLAN-006 (P0)
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let api = switchyard::Switchyard::new(facts, audit, Policy::default());

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
    let input = PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] };

    let plan = api.plan(input);
    assert_eq!(plan.actions.len(), 1, "expected one EnsureSymlink action");
}

#[test]
fn e2e_plan_003_duplicate_targets_preserved_no_dedupe() {
    // REQ-D1, E2E-PLAN-003 (P0)
    let api = switchyard::Switchyard::new(JsonlSink::default(), JsonlSink::default(), Policy::default());

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src1 = root.join("bin/new1");
    let src2 = root.join("bin/new2");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src1, b"n1").unwrap();
    std::fs::write(&src2, b"n2").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s1 = SafePath::from_rooted(root, &src1).unwrap();
    let s2 = SafePath::from_rooted(root, &src2).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();

    let input = PlanInput {
        link: vec![
            LinkRequest { source: s1, target: t.clone() },
            LinkRequest { source: s2, target: t.clone() },
        ],
        restore: vec![],
    };
    let plan = api.plan(input);
    assert_eq!(plan.actions.len(), 2, "both actions should be present (no dedupe)");
}
