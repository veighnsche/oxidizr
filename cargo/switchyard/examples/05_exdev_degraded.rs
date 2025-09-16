use switchyard::api::ApiBuilder;
use switchyard::logging::JsonlSink;
use switchyard::policy::{types::ExdevPolicy, Policy};
use switchyard::types::{ApplyMode, LinkRequest, PlanInput, SafePath};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Allow test-only env overrides inside the library to simulate EXDEV
    std::env::set_var("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES", "1");
    std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1");

    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.exdev = ExdevPolicy::DegradedFallback;

    let api = ApiBuilder::new(facts, audit, policy).build();

    let td = tempfile::tempdir()?;
    let root = td.path();
    std::fs::create_dir_all(root.join("usr/bin"))?;
    std::fs::write(root.join("usr/bin/tool"), b"old")?;
    std::fs::create_dir_all(root.join("bin"))?;
    std::fs::write(root.join("bin/new"), b"new")?;

    let source = SafePath::from_rooted(root, &root.join("bin/new"))?;
    let target = SafePath::from_rooted(root, &root.join("usr/bin/tool"))?;

    let plan = api.plan(PlanInput {
        link: vec![LinkRequest { source, target }],
        restore: vec![],
    });
    let _report = api.apply(&plan, ApplyMode::Commit)?;
    Ok(())
}
