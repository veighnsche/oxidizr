use switchyard::logging::{FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;

#[derive(Default, Clone, Debug)]
struct TestEmitter;
impl FactsEmitter for TestEmitter {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, _fields: serde_json::Value) {}
}

#[test]
fn preflight_stops_for_restore_when_no_backup_present_and_rescue_required() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.rescue.require = true; // enforce STOP behavior

    let api = switchyard::Switchyard::new(facts, audit, policy);

    // Prepare a target with no prior backup
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let tgt = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![],
        restore: vec![RestoreRequest { target: tgt }],
    });

    let pf = api.preflight(&plan).unwrap();
    assert!(
        !pf.ok,
        "preflight should STOP when restore requested but no backup artifacts present"
    );
    let joined = pf.stops.join("\n");
    assert!(
        joined.contains("no backup artifacts"),
        "stops should mention missing backup artifacts, got: {}",
        joined
    );
}
