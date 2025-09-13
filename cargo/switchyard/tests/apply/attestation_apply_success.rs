use serde_json::Value;
use switchyard::adapters::{Attestor, Signature};
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[derive(Default, Clone)]
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
struct MockAttestor;

impl Attestor for MockAttestor {
    fn sign(&self, _bundle: &[u8]) -> switchyard::types::errors::Result<Signature> {
        Ok(Signature(vec![0xAA, 0xBB, 0xCC]))
    }
    fn key_id(&self) -> String {
        "mock-key".to_string()
    }
    fn algorithm(&self) -> &'static str {
        "ed25519"
    }
}

#[test]
fn attestation_fields_present_on_success_and_masked_after_redaction() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.allow_unlocked_commit = true; // allow Commit path without LockManager
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted; // avoid gating STOP on source trust

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_attestor(Box::new(MockAttestor::default()))
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    // Setup temp tree
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"old").unwrap();

    // Build plan: single link action replacing file with symlink
    let src = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let tgt = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: src,
            target: tgt,
        }],
        restore: vec![],
    });

    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    // Find raw apply.result success and verify attestation fields exist
    let evs = facts.events.lock().unwrap();
    let mut saw_raw_attest = false;
    for (_subsystem, _event, decision, fields) in evs.iter() {
        if fields.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && decision == "success"
        {
            if let Some(att) = fields.get("attestation").and_then(|v| v.as_object()) {
                assert!(att.get("sig_alg").is_some(), "sig_alg present");
                assert!(att.get("signature").is_some(), "signature present");
                assert!(att.get("bundle_hash").is_some(), "bundle_hash present");
                assert!(att.get("public_key_id").is_some(), "public_key_id present");
                saw_raw_attest = true;
                break;
            }
        }
    }
    assert!(
        saw_raw_attest,
        "expected attestation fields in raw apply.result success"
    );

    // Verify redaction masks attestation fields
    let redacted: Vec<Value> = evs
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    let mut saw_masked = false;
    for f in redacted.iter() {
        if f.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && f.get("decision").and_then(|v| v.as_str()) == Some("success")
        {
            if let Some(att) = f.get("attestation").and_then(|v| v.as_object()) {
                assert_eq!(att.get("signature").and_then(|v| v.as_str()), Some("***"));
                assert_eq!(att.get("bundle_hash").and_then(|v| v.as_str()), Some("***"));
                assert_eq!(
                    att.get("public_key_id").and_then(|v| v.as_str()),
                    Some("***")
                );
                saw_masked = true;
                break;
            }
        }
    }
    assert!(
        saw_masked,
        "expected masked attestation fields in redacted apply.result success"
    );
}
