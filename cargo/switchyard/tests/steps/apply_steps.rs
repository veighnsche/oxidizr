use cucumber::{given, then, when};

use crate::bdd_support::facts;
use crate::bdd_world::World;
use serde_json::Value;
use switchyard::policy::types::ExdevPolicy;
use switchyard::types::plan::ApplyMode;

#[when(regex = r"^I run in real mode$")]
pub async fn when_run_real(world: &mut World) {
    world.run_real();
}

#[when(regex = r"^I run in Commit mode$")]
pub async fn when_run_commit(world: &mut World) {
    when_run_real(world).await
}

#[when(regex = r"^I run in DryRun and Commit modes$")]
pub async fn when_run_both_modes(world: &mut World) {
    world.run_both_modes();
}

#[when(regex = r"^I apply the plan$")]
pub async fn when_apply(world: &mut World) {
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    world.apply_current_plan_commit();
}

#[when(regex = r"^I run in dry-run mode$")]
pub async fn when_run_dry(world: &mut World) {
    world.run_dry_and_store();
}

#[when(regex = r"^I attempt apply in Commit mode$")]
pub async fn when_attempt_apply_commit(world: &mut World) {
    world.ensure_plan_min();
    let plan = world.plan.as_ref().unwrap();
    let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit);
}

#[then(
    regex = r"^the emitted facts for apply\.result per-action events are byte-identical after redaction$"
)]
pub async fn then_apply_result_identical(world: &mut World) {
    let dry = world.facts_dry.clone().expect("facts_dry");
    if world.facts_real.is_none() {
        when_run_real(world).await;
        world.facts_real = Some(world.all_facts());
    }
    let real = world.facts_real.clone().unwrap();
    let mut a: Vec<Value> = facts::filter_apply_result_per_action(dry)
        .into_iter()
        .map(facts::redact_and_normalize)
        .collect();
    let mut b: Vec<Value> = facts::filter_apply_result_per_action(real)
        .into_iter()
        .map(facts::redact_and_normalize)
        .collect();
    facts::sort_by_action_id(&mut a);
    facts::sort_by_action_id(&mut b);
    assert_eq!(
        a, b,
        "apply.result per-action not identical after redaction"
    );
}

#[then(
    regex = r"^the resulting facts include hash_alg=sha256 and both before_hash and after_hash$"
)]
pub async fn then_hash_fields_present(world: &mut World) {
    let facts = world.all_facts();
    let mut ok = false;
    for e in facts {
        if e.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && e.get("action_id").is_some()
            && e.get("hash_alg").and_then(|v| v.as_str()) == Some("sha256")
            && e.get("before_hash").is_some()
            && e.get("after_hash").is_some()
        {
            ok = true;
            break;
        }
    }
    assert!(
        ok,
        "missing sha256 before/after hash fields in apply.result"
    );
}

#[then(regex = r"^facts record degraded=true when policy allow_degraded_fs is enabled$")]
pub async fn then_degraded_flag(world: &mut World) {
    // enable degraded and run apply to produce fact
    world.policy.apply.exdev = ExdevPolicy::DegradedFallback;
    world.rebuild_api();
    let plan = world.plan.as_ref().unwrap();
    let _ = world
        .api
        .as_ref()
        .unwrap()
        .apply(plan, ApplyMode::Commit)
        .unwrap();
    let mut saw = false;
    for ev in world.all_facts() {
        if let Some(d) = ev.get("degraded").and_then(|v| v.as_bool()) {
            if d {
                saw = true;
                break;
            }
        }
    }
    assert!(saw, "did not observe degraded=true fact");
}

#[then(regex = r"^the operation fails with error_id=E_EXDEV when allow_degraded_fs is disabled$")]
pub async fn then_exdev_fail(world: &mut World) {
    world.policy.apply.exdev = ExdevPolicy::Fail;
    world.rebuild_api();
    world
        .env_guards
        .push(crate::bdd_support::env::EnvGuard::new(
            "SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES",
            "1",
        ));
    world
        .env_guards
        .push(crate::bdd_support::env::EnvGuard::new(
            "SWITCHYARD_FORCE_EXDEV",
            "1",
        ));
    let plan = world.plan.as_ref().unwrap();
    let _ = world
        .api
        .as_ref()
        .unwrap()
        .apply(plan, ApplyMode::Commit)
        .unwrap();
    let mut saw = false;
    for ev in world.all_facts() {
        if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_EXDEV") {
            saw = true;
            break;
        }
    }
    assert!(saw, "expected E_EXDEV in facts");
}

#[given(regex = r"^the target filesystem is read-only or noexec or immutable$")]
pub async fn given_target_fs_unsupported(world: &mut World) {
    // Simulate a policy stop by forbidding the entire temp root as an allowed target.
    let root = world.ensure_root().to_path_buf();
    world.policy.scope.forbid_paths.push(root.clone());
    world.rebuild_api();
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
}

#[when(regex = r"^I attempt to apply a plan$")]
pub async fn when_attempt_apply(world: &mut World) {
    if world.lock_path.is_none() {
        world.policy.governance.allow_unlocked_commit = true;
        world.rebuild_api();
    }
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    let plan = world.plan.as_ref().unwrap();
    let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit);
}

#[then(regex = r"^operations fail closed with a policy violation error$")]
pub async fn then_policy_violation(world: &mut World) {
    let mut saw = false;
    for ev in world.all_facts() {
        if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_POLICY") {
            saw = true;
            break;
        }
    }
    assert!(saw, "expected E_POLICY in emitted facts");
}
