use switchyard::logging::{FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[derive(Default, Clone, Debug)]
struct TestEmitter;
impl FactsEmitter for TestEmitter {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, _fields: serde_json::Value) {}
}

#[test]
fn preflight_yaml_export_matches_shape_and_writes_golden_when_requested() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback;

    let api = switchyard::Switchyard::new(facts, audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    // Setup
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/sbin")).unwrap();
    std::fs::write(root.join("bin/new1"), b"n1").unwrap();
    std::fs::write(root.join("bin/new2"), b"n2").unwrap();
    std::fs::write(root.join("usr/bin/app1"), b"o1").unwrap();
    std::fs::write(root.join("usr/sbin/app2"), b"o2").unwrap();

    let s1 = SafePath::from_rooted(root, &root.join("bin/new1")).unwrap();
    let t1 = SafePath::from_rooted(root, &root.join("usr/bin/app1")).unwrap();
    let s2 = SafePath::from_rooted(root, &root.join("bin/new2")).unwrap();
    let t2 = SafePath::from_rooted(root, &root.join("usr/sbin/app2")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![
            LinkRequest {
                source: s1,
                target: t1,
            },
            LinkRequest {
                source: s2,
                target: t2,
            },
        ],
        restore: vec![],
    });

    let pf = api.preflight(&plan).unwrap();
    let yaml = switchyard::preflight::to_yaml(&pf);
    assert!(yaml.contains("action_id"));
    assert!(yaml.contains("policy_ok"));
    assert!(yaml.contains("planned_kind"));

    if let Ok(outdir) = std::env::var("GOLDEN_OUT_DIR") {
        let path = std::path::Path::new(&outdir).join("preflight.yaml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, yaml).unwrap();
    }
}
