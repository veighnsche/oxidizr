use serde_json::Value;
use switchyard::logging::{FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput, RestoreRequest};
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
fn restore_is_invertible_with_snapshot() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.allow_unlocked_commit = true; // allow Commit without LockManager
    policy.force_untrusted_source = true; // avoid preflight STOP on non-root source in temp env
    // capture_restore_snapshot defaults to true

    let api = switchyard::Switchyard::new(facts, audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    // Prepare layout: src_new and target file
    let src_new = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src_new.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src_new, b"new").unwrap();
    std::fs::write(&tgt, b"old").unwrap();

    // Step 1: replace target with symlink to src_new (creates backup of prior file)
    let sp_src = SafePath::from_rooted(root, &src_new).unwrap();
    let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
    let plan1 = api.plan(PlanInput { link: vec![LinkRequest { source: sp_src.clone(), target: sp_tgt.clone() }], restore: vec![] });
    let _ = api.apply(&plan1, ApplyMode::Commit).unwrap();

    // Ensure target is a symlink pointing to src_new
    let md = std::fs::symlink_metadata(&tgt).unwrap();
    assert!(md.file_type().is_symlink(), "target should be symlink after replace");

    // Step 2: perform a restore from backup; snapshot of current symlink will be captured
    let plan2 = api.plan(PlanInput { link: vec![], restore: vec![RestoreRequest { target: sp_tgt.clone() }] });
    let report = api.apply(&plan2, ApplyMode::Commit).unwrap();

    // After restore, target should be a regular file again
    let md2 = std::fs::symlink_metadata(&tgt).unwrap();
    assert!(md2.file_type().is_file(), "target should be regular file after restore");

    // Step 3: compute inverse plan and apply; should restore symlink topology
    let inv = api.plan_rollback_of(&report);
    let _ = api.apply(&inv, ApplyMode::Commit).unwrap();

    let md3 = std::fs::symlink_metadata(&tgt).unwrap();
    assert!(md3.file_type().is_symlink(), "target should be symlink after inverse restore");
    let link = std::fs::read_link(&tgt).unwrap();
    assert!(link.ends_with("bin/new"), "inverse restore should re-symlink to new");
}
