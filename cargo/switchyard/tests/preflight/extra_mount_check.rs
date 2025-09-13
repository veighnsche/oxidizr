//! E2E-PREFLIGHT-008 — One extra mount check (P0)
//! Asserts a note is present for failing mount check.
//! We use `/proc` which is typically noexec → note and/or stop expected.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn one_extra_mount_check_emits_note() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.extra_mount_checks = vec![std::path::PathBuf::from("/proc")];
    // Avoid unrelated source trust stops in temp env
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let api = switchyard::Switchyard::new(facts, audit, policy);

    // Minimal plan
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
    // Look for a note mentioning not rw+exec
    let mut saw_note = false;
    for row in pf.rows.iter() {
        if let Some(notes) = row.get("notes").and_then(|v| v.as_array()) {
            if notes
                .iter()
                .any(|n| n.as_str().unwrap_or("").contains("not rw+exec"))
            {
                saw_note = true;
                break;
            }
        }
    }
    assert!(
        saw_note,
        "expected a preflight row to include a not rw+exec note"
    );
}
