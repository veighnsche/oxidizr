use std::path::PathBuf;
use switchyard::adapters::FileLockManager;
use switchyard::api::ApiBuilder;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::{ApplyMode, LinkRequest, PlanInput, SafePath};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let api = ApiBuilder::new(facts, audit, Policy::production_preset())
        .with_lock_manager(Box::new(FileLockManager::new(PathBuf::from(
            "/tmp/switchyard.lock",
        ))))
        .with_lock_timeout_ms(500)
        .build();

    let td = tempfile::tempdir()?;
    let root = td.path();
    std::fs::create_dir_all(root.join("usr/bin"))?;
    std::fs::write(root.join("usr/bin/tool"), b"old")?;
    std::fs::create_dir_all(root.join("bin"))?;
    std::fs::write(root.join("bin/new"), b"new")?;

    let source = SafePath::from_rooted(root, &root.join("bin/new"))?;
    let target = SafePath::from_rooted(root, &root.join("usr/bin/tool"))?;

    let plan = switchyard::api::Switchyard::plan(
        &api,
        PlanInput {
            link: vec![LinkRequest { source, target }],
            restore: vec![],
        },
    );
    let _report = switchyard::api::Switchyard::apply(&api, &plan, ApplyMode::Commit)?;
    Ok(())
}
