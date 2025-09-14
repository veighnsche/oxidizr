//! Preflight baseline scenarios (P0)
//! Covers:
//! - E2E-PREFLIGHT-004 (rescue not required baseline) — REQ-E2 neutral, baseline ok
//! - E2E-PREFLIGHT-010 (exec check disabled baseline)
//! - E2E-PREFLIGHT-009 (empty backup tag)
//! - E2E-PREFLIGHT-011 (coreutils tag baseline)

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

fn make_plan() -> (
    switchyard::Switchyard<JsonlSink, JsonlSink>,
    std::path::PathBuf,
) {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let api = switchyard::Switchyard::new(facts, audit, Policy::default());

    let td = tempfile::tempdir().unwrap();
    let root = td.keep();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    (api, root)
}

#[test]
#[ignore = "multi-file/architectural bug; see BUGS.md:preflight-rescue-verification"]
fn e2e_preflight_004_rescue_not_required_ok() {
    // rescue.require=false by default
    let (api, root) = make_plan();
    let s = SafePath::from_rooted(&root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(&root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });
    let pf = api.preflight(&plan).unwrap();
    assert!(pf.ok, "preflight should succeed when rescue not required");
}

#[test]
#[ignore = "multi-file/architectural bug; see BUGS.md:preflight-exec-check-handling"]
fn e2e_preflight_010_exec_check_disabled_ok() {
    let (api, root) = {
        let facts = JsonlSink::default();
        let audit = JsonlSink::default();
        let mut policy = Policy::default();
        policy.rescue.exec_check = false;
        let api = switchyard::Switchyard::new(facts, audit, policy);
        let td = tempfile::tempdir().unwrap();
        let root = td.keep();
        std::fs::create_dir_all(root.join("bin")).unwrap();
        std::fs::create_dir_all(root.join("usr/bin")).unwrap();
        std::fs::write(root.join("bin/new"), b"n").unwrap();
        std::fs::write(root.join("usr/bin/app"), b"o").unwrap();
        (api, root)
    };
    // Build plan
    let s = SafePath::from_rooted(&root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(&root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });
    let pf = api.preflight(&plan).unwrap();
    assert!(pf.ok, "preflight should succeed with exec_check disabled");
}

#[test]
#[ignore = "multi-file/architectural bug; see BUGS.md:preflight-backup-tag-handling"]
fn e2e_preflight_009_empty_backup_tag_ok() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.backup.tag = String::new();
    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });
    let pf = api.preflight(&plan).unwrap();
    assert!(pf.ok, "preflight should succeed with empty backup tag");
}

#[test]
#[ignore = "multi-file/architectural bug; see BUGS.md:preflight-coreutils-tag-handling"]
fn e2e_preflight_011_coreutils_tag_ok() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.backup.tag = "coreutils".to_string();
    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });
    let pf = api.preflight(&plan).unwrap();
    assert!(pf.ok, "preflight should succeed with coreutils tag");
}
