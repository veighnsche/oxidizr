use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{PlanInput, RestoreRequest};
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

#[test]
fn restore_emits_e_restore_failed_on_rename_error() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let dir = root.join("usr/bin");
    std::fs::create_dir_all(&dir).unwrap();

    // Create a target file and a synthetic backup file that matches the naming convention
    let target = dir.join("app");
    std::fs::write(&target, b"orig").unwrap();
    let backup_name = format!(".{}.switchyard.{}.bak", target.file_name().unwrap().to_string_lossy(), 123456789u128);
    let backup_path = dir.join(backup_name);
    std::fs::write(&backup_path, b"bak").unwrap();

    // Make the directory read-only to cause renameat to fail
    let mut p = std::fs::metadata(&dir).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    p.set_mode(0o555);
    std::fs::set_permissions(&dir, p).unwrap();

    let tgt = SafePath::from_rooted(root, &target).unwrap();
    let plan = api.plan(PlanInput { link: vec![], restore: vec![RestoreRequest { target: tgt }] });

    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    assert!(redacted.iter().any(|e| {
        e.get("stage") == Some(&Value::from("apply.result")) &&
        e.get("decision") == Some(&Value::from("failure")) &&
        e.get("error_id") == Some(&Value::from("E_RESTORE_FAILED")) &&
        e.get("exit_code") == Some(&Value::from(70))
    }), "expected E_RESTORE_FAILED failure with exit_code=70 in apply.result");
}
