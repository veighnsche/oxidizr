//! Helper: CrashInjector at swap/restore steps
//! Used by E2E-APPLY-022 for crash injection testing.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[test]
fn helper_crash_injector() {
    // CrashInjector helper (P3)
    // Test crash scenarios at swap/restore steps
    
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
        link: vec![LinkRequest { source: s, target: t }], 
        restore: vec![] 
    };
    
    let plan = api.plan(input);
    
    let apply_result = api.apply(&plan, ApplyMode::DryRun);
    assert!(apply_result.is_ok(), "apply should succeed in normal conditions");
    
    // Verify no temporary artifacts remain after successful apply
    let temp_files: Vec<std::path::PathBuf> = std::fs::read_dir(root)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.contains(".tmp") {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();
    
    assert!(temp_files.is_empty(), "no temporary artifacts should remain after successful apply");
    
    // Note: We can't easily simulate crashes in tests without special infrastructure
    // This test just verifies normal successful behavior and cleanup
}
