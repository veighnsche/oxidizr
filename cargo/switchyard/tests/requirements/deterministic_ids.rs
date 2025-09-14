//! REQ-D1 — Deterministic IDs stable across runs

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::ids::{action_id, plan_id};
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn req_d1_deterministic_ids() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s1 = SafePath::from_rooted(root, &src).unwrap();
    let t1 = SafePath::from_rooted(root, &tgt).unwrap();
    let input1 = PlanInput { link: vec![LinkRequest { source: s1, target: t1 }], restore: vec![] };
    let plan1 = api.plan(input1);

    let s2 = SafePath::from_rooted(root, &src).unwrap();
    let t2 = SafePath::from_rooted(root, &tgt).unwrap();
    let input2 = PlanInput { link: vec![LinkRequest { source: s2, target: t2 }], restore: vec![] };
    let plan2 = api.plan(input2);

    let pid1 = plan_id(&plan1);
    let pid2 = plan_id(&plan2);
    assert_eq!(pid1, pid2, "plan_id should be deterministic for identical plans");

    assert_eq!(plan1.actions.len(), plan2.actions.len());
    for (i, (a1, a2)) in plan1.actions.iter().zip(plan2.actions.iter()).enumerate() {
        let aid1 = action_id(&pid1, a1, i);
        let aid2 = action_id(&pid2, a2, i);
        assert_eq!(aid1, aid2, "action_id[{i}] stable across runs");
    }
}
