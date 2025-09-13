//! E2E-APPLY-013 — Attestation signing error tolerated; attestation omitted (REQ-O4)

use serde_json::Value;
use switchyard::adapters::{Attestor, Signature, AttestationError};
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

#[derive(Clone, Debug, Default)]
struct FailingAttestor;
impl Attestor for FailingAttestor {
    fn sign(&self, _bundle: &[u8]) -> Result<Signature, AttestationError> {
        Err(AttestationError::Signing { msg: "sign failed".to_string() })
    }
    fn key_id(&self) -> String { "mock-key".to_string() }
}

#[test]
fn attestation_error_is_tolerated_and_omitted() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.allow_unlocked_commit = true;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_attestor(Box::new(FailingAttestor))
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let redacted: Vec<Value> = facts.events.lock().unwrap().iter().map(|(_, _, _, f)| redact_event(f.clone())).collect();
    // Ensure there is an apply.result success and no attestation object
    let mut saw_success = false;
    for e in &redacted {
        if e.get("stage") == Some(&Value::from("apply.result")) && e.get("decision") == Some(&Value::from("success")) {
            saw_success = true;
            assert!(e.get("attestation").is_none(), "attestation should be omitted on signing error: {}", serde_json::to_string_pretty(e).unwrap());
        }
    }
    assert!(saw_success, "expected apply.result success event");
}
