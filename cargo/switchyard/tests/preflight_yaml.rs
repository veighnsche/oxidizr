use switchyard::policy::Policy;
use switchyard::preflight::to_yaml;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn yaml_export_includes_preservation_fields_when_present() {
    let facts = switchyard::logging::JsonlSink::default();
    let audit = switchyard::logging::JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    // Create a regular file target to exercise preservation detection
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&tgt, b"old").unwrap();

    let source = SafePath::from_rooted(root, &src).unwrap();
    let target = SafePath::from_rooted(root, &tgt).unwrap();

    let plan = api.plan(PlanInput { link: vec![LinkRequest { source, target }], restore: vec![] });
    let report = api.preflight(&plan).unwrap();

    let out = to_yaml(&report);
    assert!(out.contains("preservation:"), "YAML should include preservation section: {}", out);
    assert!(out.contains("preservation_supported"), "YAML should include preservation_supported flag: {}", out);
}
