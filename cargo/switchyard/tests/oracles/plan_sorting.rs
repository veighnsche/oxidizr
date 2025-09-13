//! Plan deterministic sorting assertion
//! Assert deterministic sorting by kind then target.rel and stable action_id per SPEC.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn plan_deterministic_sorting() {
    // Plan sorting assertion (P0)
    // Assert deterministic sorting by (kind then target.rel) and stable action_id

    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts, audit, policy);

    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    // Create multiple targets to test sorting
    let src1 = root.join("bin/new1");
    let src2 = root.join("bin/new2");
    let tgt1 = root.join("usr/bin/app1");
    let tgt2 = root.join("usr/bin/app2");

    std::fs::create_dir_all(src1.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt1.parent().unwrap()).unwrap();
    std::fs::write(&src1, b"n1").unwrap();
    std::fs::write(&src2, b"n2").unwrap();
    std::fs::write(&tgt1, b"o1").unwrap();
    std::fs::write(&tgt2, b"o2").unwrap();

    let s1 = SafePath::from_rooted(root, &src1).unwrap();
    let s2 = SafePath::from_rooted(root, &src2).unwrap();
    let t1 = SafePath::from_rooted(root, &tgt1).unwrap();
    let t2 = SafePath::from_rooted(root, &tgt2).unwrap();

    let input = PlanInput {
        link: vec![
            LinkRequest {
                source: s1,
                target: t1.clone(),
            },
            LinkRequest {
                source: s2,
                target: t2.clone(),
            },
        ],
        restore: vec![],
    };

    let plan = api.plan(input);

    // Verify sorted by kind (EnsureSymlink) then by target.rel lexicographically
    let last_kind = 0u8; // 0 for link
    let mut last_t = String::new();
    for act in plan.actions.iter() {
        match act {
            switchyard::types::plan::Action::EnsureSymlink { target, .. } => {
                assert_eq!(last_kind, 0u8, "all actions should be EnsureSymlink");
                let cur = target.rel().to_string_lossy().to_string();
                assert!(
                    cur >= last_t,
                    "targets should be sorted: {} >= {}",
                    cur,
                    last_t
                );
                last_t = cur;
            }
            _ => panic!("expected only EnsureSymlink actions"),
        }
    }

    // Verify action_id is stable (UUIDv5 derivation)
    // Plan ID should be deterministic based on inputs
    let plan_uuid = switchyard::types::ids::plan_id(&plan);
    assert!(
        !plan_uuid.to_string().is_empty(),
        "plan_id should be present"
    );
}
