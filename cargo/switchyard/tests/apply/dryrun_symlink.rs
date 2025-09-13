//! E2E-APPLY-001 — DryRun symlink ensure (REQ-D2, REQ-O1, REQ-TOCTOU1)
//! - Assert ApplyReport.errors=[]
//! - Assert per-action facts have TS_ZERO after redaction and after_kind="symlink"
//! - Deterministic temp roots; SafePath-only

use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
}
impl FactsEmitter for TestEmitter {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
        self.events.lock().unwrap().push((
            subsystem.to_string(),
            event.to_string(),
            decision.to_string(),
            fields,
        ));
    }
}

#[test]
fn dryrun_symlink_emits_symlink_after_kind_and_no_errors() {
    // REQ-D2, REQ-O1, REQ-TOCTOU1
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted; // avoid gating on temp files
    policy.apply.override_preflight = true; // E2E-APPLY-001 requires override_preflight=true

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    // Layout under temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let input = PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] };

    let plan = api.plan(input);
    let _ = api.preflight(&plan).unwrap();
    let report = api.apply(&plan, ApplyMode::DryRun).unwrap();
    assert!(report.errors.is_empty(), "DryRun should not record apply errors");

    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    assert!(redacted.iter().any(|e| {
        e.get("stage") == Some(&Value::from("apply.result"))
            && e.get("decision") == Some(&Value::from("success"))
            && e.get("after_kind") == Some(&Value::from("symlink"))
            && e.get("ts") == Some(&Value::from("1970-01-01T00:00:00Z"))
    }), "expected apply.result success with after_kind=symlink and TS_ZERO");
}
