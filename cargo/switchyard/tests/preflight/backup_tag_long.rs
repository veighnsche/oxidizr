//! E2E-PREFLIGHT-007 — Long backup tag annotation (P0)
//! Asserts that preflight rows carry the backup_tag when set to a long value.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn preflight_rows_carry_long_backup_tag() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.backup.tag = "x".repeat(256);
    let tag = policy.backup.tag.clone();
    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    let pf = api.preflight(&plan).unwrap();
    assert!(!pf.rows.is_empty(), "expected preflight rows");
    assert!(pf.rows.iter().any(|r| r.get("backup_tag").and_then(|v| v.as_str()) == Some(tag.as_str())),
        "preflight rows should carry backup_tag");
}
