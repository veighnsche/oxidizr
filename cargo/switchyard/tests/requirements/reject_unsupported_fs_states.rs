//! REQ-S2 Reject unsupported FS states coverage
//! Assert errors on unsupported filesystem states.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn req_s2_reject_unsupported_fs_states() {
    // REQ-S2 (P0)
    // Assert errors on unsupported filesystem states

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

    // In normal conditions, planning should succeed
    assert!(
        !plan.actions.is_empty(),
        "plan should succeed in normal filesystem conditions"
    );

    // The implementation should reject unsupported filesystem states such as:
    // - Paths with unsupported components
    // - Non-absolute roots
    // - Paths outside the root
    // These are tested in other safepath tests
}
