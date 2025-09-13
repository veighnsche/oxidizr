//! E2E-PRUNE-001 — Count limit min=0; newest retained (REQ-PN1)

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::safepath::SafePath;

fn make_snaps(tgt: &std::path::Path, tag: &str, n: usize) {
    for _ in 0..n {
        let _ = switchyard::fs::backup::create_snapshot(tgt, tag);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

#[test]
fn prune_min0_keeps_only_newest() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.retention_count_limit = Some(0); // min=0 -> clamp to retain newest only
    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&tgt, b"old").unwrap();

    make_snaps(&tgt, switchyard::constants::DEFAULT_BACKUP_TAG, 4);

    let sp = SafePath::from_rooted(root, &tgt).unwrap();
    let res = api.prune_backups(&sp).unwrap();
    assert_eq!(res.retained_count, 1, "min=0 should retain only newest");
    assert_eq!(res.pruned_count + res.retained_count, 4);
}
