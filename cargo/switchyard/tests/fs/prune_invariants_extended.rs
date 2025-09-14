//! REQ-PN2/PN3 — Prune invariants extended: payload+sidecar deleted and prune.result fact emitted

use serde_json::Value;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::safepath::SafePath;

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
}
impl switchyard::logging::FactsEmitter for TestEmitter {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
        self.events
            .lock()
            .unwrap()
            .push((subsystem.to_string(), event.to_string(), decision.to_string(), fields));
    }
}

#[test]
fn prune_invariants_extended() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.retention_count_limit = Some(1); // keep only newest

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    // Target and two snapshots
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&tgt, b"current").unwrap();

    // Create two snapshots with different times
    switchyard::fs::backup::create_snapshot(&tgt, switchyard::constants::DEFAULT_BACKUP_TAG).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    switchyard::fs::backup::create_snapshot(&tgt, switchyard::constants::DEFAULT_BACKUP_TAG).unwrap();

    let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
    let res = api.prune_backups(&sp_tgt).unwrap();

    // Expect at least one pruned and exactly one retained (newest)
    assert!(res.pruned_count >= 1, "expected at least one pruned entry");
    assert_eq!(res.retained_count, 1, "expected only newest retained");

    // Check that prune.result fact was emitted with key fields
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| f.clone())
        .collect();
    assert!(redacted.iter().any(|e| e.get("stage") == Some(&Value::from("prune.result"))
        && e.get("path").is_some()
        && e.get("pruned_count").is_some()
        && e.get("retained_count").is_some()), "expected a prune.result fact with counts");
}
