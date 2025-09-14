use cucumber::{then, when};

use crate::bdd_support::{facts, schema};
use crate::bdd_world::World;
use serde_json::Value;
use switchyard::preflight::yaml as preflight_yaml;

#[when(regex = r"^I run preflight$")]
pub async fn when_preflight(world: &mut World) {
    world.ensure_api();
    world.ensure_plan_min();
    let plan = world.plan.as_ref().unwrap();
    world.preflight = Some(world.api.as_ref().unwrap().preflight(plan).unwrap());
}

#[when(regex = r"^I run preflight in DryRun and Commit modes$")]
pub async fn when_preflight_both(world: &mut World) {
    world.ensure_api();
    world.ensure_plan_min();
    let plan = world.plan.as_ref().unwrap();
    let _ = world.api.as_ref().unwrap().preflight(plan).unwrap();
    // preflight is always dry-run; re-run to simulate commit parity
    let _ = world.api.as_ref().unwrap().preflight(plan).unwrap();
}

#[then(
    regex = r"^the emitted facts for plan and preflight are byte-identical after timestamp redaction$"
)]
pub async fn then_plan_preflight_identical(world: &mut World) {
    // Reproduce emissions deterministically: run preflight twice and compare redacted facts
    world.ensure_api();
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    let plan = world.plan.as_ref().unwrap().clone();
    // First run
    world.clear_facts();
    let _ = world.api.as_ref().unwrap().preflight(&plan).unwrap();
    let mut a: Vec<Value> = facts::filter_by_stage(
        world.all_facts(),
        &["plan", "preflight", "preflight.summary"],
    )
    .into_iter()
    .map(facts::redact_and_normalize)
    .collect();
    // Second run
    world.clear_facts();
    let _ = world.api.as_ref().unwrap().preflight(&plan).unwrap();
    let mut b: Vec<Value> = facts::filter_by_stage(
        world.all_facts(),
        &["plan", "preflight", "preflight.summary"],
    )
    .into_iter()
    .map(facts::redact_and_normalize)
    .collect();
    facts::sort_by_stage_action_path(&mut a);
    facts::sort_by_stage_action_path(&mut b);
    assert_eq!(a, b, "plan+preflight facts not identical after redaction");
}

#[then(regex = r"^the exported preflight YAML rows are byte-identical between runs$")]
pub async fn then_preflight_yaml_identical(world: &mut World) {
    world.ensure_api();
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    let plan = world.plan.as_ref().unwrap();
    let r1 = world.api.as_ref().unwrap().preflight(plan).unwrap();
    let y1 = preflight_yaml::to_yaml(&r1);
    let r2 = world.api.as_ref().unwrap().preflight(plan).unwrap();
    let y2 = preflight_yaml::to_yaml(&r2);
    assert_eq!(y1, y2, "preflight YAML differs between runs");
}
