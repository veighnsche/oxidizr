//! E2E-APPLY-002 — Commit happy path (REQ-L1, REQ-O1)
//! Locking Required with lock manager present; smoke Off. Expect success.

use serde_json::Value;
use switchyard::adapters::FileLockManager;
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
fn commit_happy_path_succeeds_and_emits_success() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.governance.locking = switchyard::policy::types::LockingPolicy::Required;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    // Temp root and lock path
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let lock_path = root.join("switchyard.lock");

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    // Layout
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });
    let _pf = api.preflight(&plan).unwrap();
    let report = api.apply(&plan, ApplyMode::Commit).unwrap();

    assert!(report.errors.is_empty(), "no errors on happy path");

    // Ensure apply.result success and no error_id
    let redacted: Vec<Value> = facts.events.lock().unwrap().iter().map(|(_, _, _, f)| redact_event(f.clone())).collect();
    assert!(redacted.iter().any(|e| e.get("stage") == Some(&Value::from("apply.result")) && e.get("decision") == Some(&Value::from("success"))),
        "expected apply.result success");
    assert!(!redacted.iter().any(|e| e.get("stage") == Some(&Value::from("apply.result")) && e.get("error_id").is_some()),
        "apply.result should not carry error_id on success");
}
