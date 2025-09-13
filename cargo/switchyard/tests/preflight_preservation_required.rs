use switchyard::logging::{FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[derive(Default, Clone)]
struct TestEmitter;
impl FactsEmitter for TestEmitter {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, _fields: serde_json::Value) {}
}

#[test]
fn preflight_stops_when_preservation_required_and_unsupported() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.durability.preservation = switchyard::policy::types::PreservationPolicy::RequireBasic;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted; // avoid source trust STOP

    // Use allow_roots to avoid scope STOPs for this test
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    // Intentionally do NOT create the target path, so preservation_supported=false

    let api = switchyard::Switchyard::new(facts, audit, policy);

    let src = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let tgt = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();

    let mut pol = switchyard::policy::Policy::default();
    pol.durability.preservation = switchyard::policy::types::PreservationPolicy::RequireBasic;
    pol.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;
    pol.scope.allow_roots.push(root.join("usr/bin"));
    let api = switchyard::Switchyard::new(TestEmitter::default(), JsonlSink::default(), pol);

    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: src,
            target: tgt,
        }],
        restore: vec![],
    });

    let pf = api.preflight(&plan).unwrap();
    assert!(
        !pf.ok,
        "preflight should STOP when preservation is required but unsupported"
    );
    let stops = pf.stops.join("\n");
    assert!(
        stops.contains("preservation unsupported for target"),
        "expected STOP reason to mention preservation unsupported, got: {}",
        stops
    );
}
