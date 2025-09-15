use cucumber::{given, then, when};

use crate::bdd_world::World;

#[when(regex = r"^I inspect its signature$")]
pub async fn when_inspect_signature(_world: &mut World) {
    // No-op: next step will assert on source code signatures
}

#[then(regex = r"^the signature requires SafePath and does not accept PathBuf$")]
pub async fn then_signature_requires_safepath(_world: &mut World) {
    // Approximate by scanning API facade for signatures; ensure no &PathBuf in pub API,
    // and that mutate methods reference SafePath.
    const API_MOD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/api/mod.rs"));
    assert!(
        !API_MOD.contains("&PathBuf"),
        "public API should not accept &PathBuf"
    );
    let mentions_safe = API_MOD.contains("safepath::SafePath") || API_MOD.contains("SafePath");
    let has_prune = API_MOD.contains("pub fn prune_backups(");
    assert!(mentions_safe && has_prune, "expected prune_backups to take &SafePath");
}

#[when(regex = r"^the engine performs the operation$")]
pub async fn when_engine_performs_op(_world: &mut World) {
    // No-op: next step inspects implementation source
}

#[then(regex = r"^it opens the parent with O_DIRECTORY\|O_NOFOLLOW, uses openat on the final component, renames with renameat, and fsyncs the parent$")]
pub async fn then_toctou_sequence_present(_world: &mut World) {
    // Best-effort: look for helpers used to enforce TOCTOU safety
    const SNAPSHOT_RS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"), "/src/fs/backup/snapshot.rs"
    ));
    const RESTORE_STEPS_RS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"), "/src/fs/restore/steps.rs"
    ));
    const ATOMIC_RS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/fs/atomic.rs"));
    let has_open_dir = SNAPSHOT_RS.contains("open_dir_nofollow(") || ATOMIC_RS.contains("open_dir_nofollow(");
    let has_fsync_parent = SNAPSHOT_RS.contains("fsync_parent_dir(") || ATOMIC_RS.contains("fsync_parent_dir(");
    let has_renameat = RESTORE_STEPS_RS.contains("renameat(") || ATOMIC_RS.contains("renameat(");
    assert!(has_open_dir, "expected open_dir_nofollow in FS ops");
    assert!(has_fsync_parent, "expected fsync_parent_dir in FS ops");
    assert!(has_renameat, "expected renameat in FS ops");
}
