use switchyard::api::ApiBuilder;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::{ApplyMode, LinkRequest, PlanInput, RestoreRequest, SafePath};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let api = ApiBuilder::new(facts, audit, Policy::default()).build();

    let td = tempfile::tempdir()?;
    let root = td.path();
    std::fs::create_dir_all(root.join("usr/bin"))?;
    std::fs::write(root.join("usr/bin/tool"), b"old")?;
    std::fs::create_dir_all(root.join("bin"))?;
    std::fs::write(root.join("bin/new"), b"new")?;

    let source = SafePath::from_rooted(root, &root.join("bin/new"))?;
    let target = SafePath::from_rooted(root, &root.join("usr/bin/tool"))?;
    // Choose a separate path that has no backup to force a restore failure
    let missing_target = SafePath::from_rooted(root, &root.join("usr/bin/other"))?;

    // Plan with a symlink replacement followed by a restore that is expected to fail
    // by targeting a different path that has no backup artifacts.
    let plan = PlanInput {
        link: vec![LinkRequest {
            source,
            target: target.clone(),
        }],
        restore: vec![RestoreRequest {
            target: missing_target.clone(),
        }],
    };
    let plan = api.plan(plan);

    let report = api.apply(&plan, ApplyMode::Commit)?;
    if !report.errors.is_empty() && report.rolled_back {
        eprintln!("Rollback occurred as expected: {:?}", report.errors);
    }

    // Demonstrate planning rollback of a previous report (even when manual):
    let _rollback_plan = api.plan_rollback_of(&report);
    Ok(())
}
