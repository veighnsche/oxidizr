//! Compile-only public API surface smoke test.
//! Ensures typical consumer imports compile and simple flows run.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;
use switchyard::{adapters::DefaultSmokeRunner, adapters::FileLockManager, Switchyard};

#[test]
fn public_api_compiles_and_runs_dry() {
    // Construct API
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.allow_unlocked_commit = true; // allow Commit without lock manager in tests

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let lock_path = root.join("switchyard.lock");

    let api = Switchyard::new(facts, audit, policy)
        .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
        .with_smoke_runner(Box::new(DefaultSmokeRunner::default()));

    // Plan a simple link action under a temp root using SafePath
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();

    let source = SafePath::from_rooted(root, &src).unwrap();
    let target = SafePath::from_rooted(root, &tgt).unwrap();
    let input = PlanInput {
        link: vec![LinkRequest { source, target }],
        restore: vec![],
    };

    let plan = api.plan(input);
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();
}
