//! E2E-PREFLIGHT-006 — Extra mount checks (5) (REQ-O1)
//! Assert notes present in preflight rows.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn e2e_preflight_006_extra_mount_checks_five() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    // Set 5 extra mount checks
    policy.apply.extra_mount_checks = vec![
        "/tmp".to_string(),
        "/var".to_string(),
        "/home".to_string(),
        "/opt".to_string(),
        "/usr/local".to_string(),
    ];
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
    
    // Check that extra mount checks are present in the notes
    assert!(pf.rows.len() > 0, "preflight should have rows");
    let has_mount_notes = pf.rows.iter().any(|row| {
        row.notes.iter().any(|note| note.contains("mount"))
    });
    assert!(has_mount_notes, "preflight rows should contain mount check notes");
}
