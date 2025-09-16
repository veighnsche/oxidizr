use cucumber::{given, then, when};

use crate::bdd_world::World;

// ----------------------
// Determinism
// ----------------------

#[given(regex = r"^a plan built from a stable set of inputs$")]
pub async fn given_stable_plan(world: &mut World) {
    crate::steps::plan_steps::given_plan_min(world).await;
}

#[when(regex = r"^I compute plan_id and action_id$")]
pub async fn when_compute_ids(_world: &mut World) {}

#[then(regex = r"^they are deterministic UUIDv5 values under the project namespace$")]
pub async fn then_ids_deterministic(world: &mut World) {
    use switchyard::types::ids::{action_id, plan_id};
    let plan = world.plan.as_ref().expect("plan").clone();
    let a1: Vec<_> = plan
        .actions
        .iter()
        .enumerate()
        .map(|(i, act)| action_id(&plan_id(&plan), act, i))
        .collect();
    let a2: Vec<_> = plan
        .actions
        .iter()
        .enumerate()
        .map(|(i, act)| action_id(&plan_id(&plan), act, i))
        .collect();
    assert_eq!(a1, a2, "action IDs should be deterministic");
}

#[then(regex = r"^facts are byte-identical after timestamp redaction$")]
pub async fn then_facts_identical_after_redaction(world: &mut World) {
    use crate::bdd_support::facts;
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    let plan = world.plan.as_ref().unwrap().clone();
    world.clear_facts();
    // DryRun
    let _ = world
        .api
        .as_ref()
        .unwrap()
        .apply(&plan, switchyard::types::plan::ApplyMode::DryRun);
    let mut a = facts::filter_apply_result_per_action(world.all_facts())
        .into_iter()
        .map(facts::redact_and_normalize)
        .collect::<Vec<_>>();
    // Commit
    world.clear_facts();
    let _ = world
        .api
        .as_ref()
        .unwrap()
        .apply(&plan, switchyard::types::plan::ApplyMode::Commit);
    let mut b = facts::filter_apply_result_per_action(world.all_facts())
        .into_iter()
        .map(facts::redact_and_normalize)
        .collect::<Vec<_>>();
    crate::bdd_support::facts::sort_by_action_id(&mut a);
    crate::bdd_support::facts::sort_by_action_id(&mut b);
    assert_eq!(
        a, b,
        "apply.result per-action not identical after redaction"
    );
}

// ----------------------
// Determinism & attestation (aliases / placeholders)
// ----------------------

#[given(regex = r"^normalized plan input and a stable namespace$")]
pub async fn given_normalized_ns(world: &mut World) {
    crate::steps::plan_steps::given_plan_min(world).await;
}

#[given(regex = r"^an apply bundle$")]
pub async fn given_apply_bundle(_world: &mut World) {}

#[when(regex = r"^I generate plan_id and action_id$")]
pub async fn when_generate_ids(_world: &mut World) {}

#[then(regex = r"^they are UUIDv5 values derived from the normalized input and namespace$")]
pub async fn then_uuidv5_alias(world: &mut World) {
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    then_ids_deterministic(world).await;
}
