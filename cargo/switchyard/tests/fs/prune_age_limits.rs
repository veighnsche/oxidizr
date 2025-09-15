//! Age-based pruning exact limits tests (P0)
//! Implements E2E-PRUNE-009 (365d max) and E2E-PRUNE-010 (1s min)

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use switchyard::types::safepath::SafePath;

#[test]
fn prune_max_age_365d_prunes_older() {
    // Layout under temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    let target = root.join("usr/bin/app");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();

    // Compose backup file names with explicit timestamps in the filename
    let name = target.file_name().and_then(|s| s.to_str()).unwrap();
    let tag = switchyard::constants::DEFAULT_BACKUP_TAG;
    let parent = target.parent().unwrap();

    let now_ms: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // Three backups: 400d old (should prune), 200d old (keep), newest (keep)
    let days = |d: u64| -> u128 { Duration::from_secs(d * 24 * 60 * 60).as_millis() };
    let ts_old = now_ms.saturating_sub(days(400));
    let ts_mid = now_ms.saturating_sub(days(200));
    let ts_new = now_ms;

    let mk = |ts: u128| -> std::path::PathBuf {
        let p = parent.join(format!(".{name}.{tag}.{ts}.bak"));
        std::fs::write(&p, b"x").unwrap();
        // Create a sidecar alongside (content irrelevant for prune)
        let sc = std::path::PathBuf::from(format!("{}{}.meta.json", p.display(), ""));
        std::fs::write(sc, b"{}\n").unwrap();
        p
    };

    let old_b = mk(ts_old);
    let mid_b = mk(ts_mid);
    let new_b = mk(ts_new);

    // Run prune with age_limit = 365d, no count limit
    let sp_tgt = SafePath::from_rooted(root, &target).unwrap();
    let res = switchyard::fs::backup::prune::prune_backups(
        &sp_tgt,
        tag,
        None,
        Some(Duration::from_secs(365 * 24 * 60 * 60)),
    )
    .unwrap();

    // Expect exactly the oldest to be pruned; newest always retained
    assert_eq!(res.pruned_count, 1, "expected one pruned entry");
    assert_eq!(res.retained_count, 2, "expected two retained entries");

    assert!(!old_b.exists(), "old backup should be pruned");
    assert!(mid_b.exists(), "mid backup should be retained");
    assert!(new_b.exists(), "newest backup should be retained");
}

#[test]
fn prune_min_age_1s_prunes_older() {
    // Layout under temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    let target = root.join("usr/bin/app");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();

    let name = target.file_name().and_then(|s| s.to_str()).unwrap();
    let tag = switchyard::constants::DEFAULT_BACKUP_TAG;
    let parent = target.parent().unwrap();

    let now_ms: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // Three backups: 2000ms old (prune), 100ms old (keep), newest (keep)
    let ts_old = now_ms.saturating_sub(2000);
    let ts_mid = now_ms.saturating_sub(100);
    let ts_new = now_ms;

    let mk = |ts: u128| -> std::path::PathBuf {
        let p = parent.join(format!(".{name}.{tag}.{ts}.bak"));
        std::fs::write(&p, b"x").unwrap();
        // Sidecar (content irrelevant)
        let sc = std::path::PathBuf::from(format!("{}{}.meta.json", p.display(), ""));
        std::fs::write(sc, b"{}\n").unwrap();
        p
    };

    let old_b = mk(ts_old);
    let mid_b = mk(ts_mid);
    let new_b = mk(ts_new);

    let sp_tgt = SafePath::from_rooted(root, &target).unwrap();
    let res = switchyard::fs::backup::prune::prune_backups(
        &sp_tgt,
        tag,
        None,
        Some(Duration::from_millis(1000)),
    )
    .unwrap();

    assert_eq!(res.pruned_count, 1, "expected one pruned entry");
    assert_eq!(res.retained_count, 2, "expected two retained entries");

    assert!(!old_b.exists(), "old backup should be pruned");
    assert!(mid_b.exists(), "mid backup should be retained");
    assert!(new_b.exists(), "newest backup should be retained");
}
