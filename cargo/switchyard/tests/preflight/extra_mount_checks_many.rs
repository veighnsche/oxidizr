//! E2E-PREFLIGHT-006 — Extra mount checks (5) (P1)
//! Provide 5 mount paths (some intentionally non-existent) to trigger notes.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn extra_mount_checks_many_emit_notes() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.extra_mount_checks = vec![
        std::path::PathBuf::from("/proc"),
        std::path::PathBuf::from("/sys"),
        std::path::PathBuf::from("/definitely-missing-a"),
        std::path::PathBuf::from("/definitely-missing-b"),
        std::path::PathBuf::from("/definitely-missing-c"),
    ];
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
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    let pf = api.preflight(&plan).unwrap();
    // At least one row should include a note mentioning not rw+exec
    let mut saw = false;
    for row in pf.rows.iter() {
        if let Some(notes) = row.get("notes").and_then(|v| v.as_array()) {
            if notes.iter().any(|n| n.as_str().unwrap_or("").contains("not rw+exec")) {
                saw = true; break;
            }
        }
    }
    assert!(saw, "expected at least one not rw+exec note from extra mount checks");
}
