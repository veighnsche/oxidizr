use serde_json::Value;
use switchyard::logging::{FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::safepath::SafePath;
use switchyard::constants::DEFAULT_BACKUP_TAG;

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>> ,
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

fn make_snapshots(tgt: &std::path::Path, tag: &str, n: usize) {
    for _ in 0..n {
        let _ = switchyard::fs::backup::create_snapshot(tgt, tag);
        std::thread::sleep(std::time::Duration::from_millis(3));
    }
}

#[test]
fn prune_by_count_keeps_newest_and_limits_total() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.retention_count_limit = Some(2);
    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    // Start with a regular file so payload exists
    std::fs::write(&tgt, b"old").unwrap();

    // Create 3 snapshots
    make_snapshots(&tgt, DEFAULT_BACKUP_TAG, 3);

    // Prune
    let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
    let res = api.prune_backups(&sp_tgt).expect("prune ok");
    // Expect 1 pruned (3 -> keep newest + one more = 2 retained)
    assert_eq!(res.pruned_count + res.retained_count, 3);
    assert_eq!(res.retained_count, 2);

    // Verify event emitted
    let evs = facts.events.lock().unwrap();
    assert!(evs.iter().any(|(_, event, decision, _)| event == "prune.result" && decision == "success"));
}

#[test]
fn prune_by_age_prunes_old_backups() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    // Age limit very small so earlier snapshots become old
    policy.retention_age_limit = Some(std::time::Duration::from_millis(1));
    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&tgt, b"old").unwrap();

    // Create 3 snapshots
    make_snapshots(&tgt, DEFAULT_BACKUP_TAG, 3);

    // Sleep so last snapshot also ages beyond threshold
    std::thread::sleep(std::time::Duration::from_millis(5));

    let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
    let res = api.prune_backups(&sp_tgt).expect("prune ok");

    assert!(res.pruned_count >= 2, "expected at least two old backups pruned, got {}", res.pruned_count);
    assert!(facts.events.lock().unwrap().iter().any(|(_, ev, _, _)| ev == "prune.result"));
}
