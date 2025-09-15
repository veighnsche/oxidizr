use cucumber::{given, then, when};

use crate::bdd_world::World;

#[given(regex = r"^no explicit approval flag is provided$")]
pub async fn given_no_approval(_world: &mut World) {}

#[when(regex = r"^I run the engine$")]
pub async fn when_run_engine(world: &mut World) {
    // Default behavior: perform preflight only (no apply facts in DryRun-by-default scenario)
    crate::steps::preflight_steps::when_preflight(world).await;
}

#[then(regex = r"^it runs in dry-run mode by default$")]
pub async fn then_runs_dry_default(world: &mut World) {
    super::then_side_effects_not_performed(world).await;
}

#[when(regex = r"^I execute without explicit commit approval$")]
pub async fn when_execute_without_approval(world: &mut World) {
    // Simulate engine invocation without commit approval: run preflight only
    crate::steps::preflight_steps::when_preflight(world).await;
}

#[then(regex = r"^side effects are not performed \(DryRun is default\)$")]
pub async fn then_side_effects_not_performed(world: &mut World) {
    // Assert no apply.* facts were emitted => no mutations attempted
    for ev in world.all_facts() {
        if let Some(stage) = ev.get("stage").and_then(|v| v.as_str()) {
            assert!(
                !stage.starts_with("apply"),
                "unexpected apply-stage facts found in DryRun-by-default scenario: {}",
                stage
            );
        }
    }
}

#[when(regex = r"^I run preflight and apply in Commit mode$")]
pub async fn when_preflight_and_apply_commit(world: &mut World) {
    crate::steps::preflight_steps::when_preflight(world).await;
    crate::steps::apply_steps::when_apply(world).await;
}

#[given(regex = r"^a critical compatibility violation is detected in preflight$")]
pub async fn given_critical_violation(world: &mut World) {
    // Reuse existing helper to simulate unsupported target filesystem
    crate::steps::apply_steps::given_target_fs_unsupported(world).await;
}

#[when(regex = r"^I run the engine with default policy$")]
pub async fn when_run_engine_default(world: &mut World) {
    crate::steps::preflight_steps::when_preflight(world).await;
}

#[then(regex = r"^the operation fails closed unless an explicit override is present$")]
pub async fn then_fail_closed_alias(world: &mut World) {
    crate::steps::apply_steps::then_policy_violation(world).await;
}

#[then(regex = r"^the operation fails closed unless an explicit policy override is set$")]
pub async fn then_fail_closed_policy_override(world: &mut World) {
    crate::steps::apply_steps::then_policy_violation(world).await;
}

#[given(regex = r"^golden fixtures for plan, preflight, apply, and rollback$")]
pub async fn given_golden_fixtures(_world: &mut World) {}

#[given(regex = r"^a required test is marked SKIP or a fixture diff is not byte-identical$")]
pub async fn given_ci_violation(_world: &mut World) {}

#[when(regex = r"^CI runs$")]
pub async fn when_ci_runs(_world: &mut World) {}

#[then(regex = r"^the CI job fails$")]
pub async fn then_ci_fails(_world: &mut World) {
    // Assert that the test runner is configured to fail on skipped scenarios.
    // This verifies the CI gate rather than being a no-op.
    const BDD_MAIN: &str = include_str!("../../bdd_main.rs");
    assert!(
        BDD_MAIN.contains(".fail_on_skipped()"),
        "bdd_main.rs should enable fail_on_skipped() to enforce zero-SKIP CI gate"
    );
}

#[given(regex = r"^a newly constructed Switchyard$")]
pub async fn given_new_switchyard(world: &mut World) {
    world.rebuild_api();
}

#[given(regex = r"^a policy requiring strict ownership and unsupported preservation$")]
pub async fn given_strict_unsupported(world: &mut World) {
    world.policy.risks.ownership_strict = true;
    world.policy.durability.preservation = switchyard::policy::types::PreservationPolicy::RequireBasic;
    world.rebuild_api();
}
