use cucumber::{given, then, when};

use crate::bdd_world::World;
use switchyard::api::Switchyard;

#[given(regex = r"^a Switchyard with SmokePolicy Require$")]
pub async fn given_switchyard_smoke_require(world: &mut World) {
    crate::steps::plan_steps::given_smoke(world).await;
}

#[given(regex = r"^a failing SmokeTestRunner$")]
pub async fn given_failing_smoke_runner(world: &mut World) {
    #[derive(Debug, Default)]
    struct Failing;
    impl switchyard::adapters::SmokeTestRunner for Failing {
        fn run(
            &self,
            _plan: &switchyard::types::plan::Plan,
        ) -> Result<(), switchyard::adapters::smoke::SmokeFailure> {
            Err(switchyard::adapters::smoke::SmokeFailure)
        }
    }
    let api = Switchyard::builder(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    )
    .with_smoke_runner(Box::new(Failing))
    .build();
    world.api = Some(api);
    world.smoke_runner = Some(crate::bdd_world::SmokeRunnerKind::Failing);
}

#[then(regex = r"^the smoke suite runs and detects the failure$")]
pub async fn then_smoke_detects_failure(world: &mut World) {
    let mut saw = false;
    for ev in world.all_facts() {
        if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_SMOKE") {
            saw = true;
            break;
        }
        if let Some(arr) = ev.get("summary_error_ids").and_then(|v| v.as_array()) {
            if arr.iter().any(|x| x.as_str() == Some("E_SMOKE")) {
                saw = true;
                break;
            }
        }
    }
    assert!(saw, "expected E_SMOKE in facts");
}

#[then(regex = r"^automatic rollback occurs unless policy explicitly disables it$")]
pub async fn then_auto_rollback_occurs(world: &mut World) {
    let mut saw_rb = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("rollback") {
            saw_rb = true;
            break;
        }
    }
    assert!(saw_rb, "expected rollback events after smoke failure");
}

#[given(regex = r"^a configured SmokeTestRunner$")]
pub async fn given_configured_smoke_runner(world: &mut World) {
    let api = Switchyard::builder(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    )
    .with_smoke_runner(Box::new(switchyard::adapters::DefaultSmokeRunner))
    .build();
    world.api = Some(api);
}

#[given(regex = r"^auto_rollback is enabled$")]
pub async fn given_auto_rollback_enabled(world: &mut World) {
    world.policy.governance.smoke = switchyard::policy::types::SmokePolicy::Require {
        auto_rollback: true,
    };
    // Preserve any configured runner when rebuilding API
    let mut builder = Switchyard::builder(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    );
    if let Some(kind) = world.smoke_runner {
        match kind {
            crate::bdd_world::SmokeRunnerKind::Default => {
                builder =
                    builder.with_smoke_runner(Box::new(switchyard::adapters::DefaultSmokeRunner));
            }
            crate::bdd_world::SmokeRunnerKind::Failing => {
                #[derive(Debug, Default)]
                struct Failing;
                impl switchyard::adapters::SmokeTestRunner for Failing {
                    fn run(
                        &self,
                        _plan: &switchyard::types::plan::Plan,
                    ) -> Result<(), switchyard::adapters::smoke::SmokeFailure> {
                        Err(switchyard::adapters::smoke::SmokeFailure)
                    }
                }
                builder = builder.with_smoke_runner(Box::new(Failing));
            }
        }
    }
    world.api = Some(builder.build());
}

#[given(regex = r"^at least one smoke command will fail with a non-zero exit$")]
pub async fn given_smoke_command_will_fail(world: &mut World) {
    given_failing_smoke_runner(world).await;
    // Remember the failing runner kind so apply_current_plan_commit reattaches it
    world.smoke_runner = Some(crate::bdd_world::SmokeRunnerKind::Failing);
    // Ensure smoke is required so failures map to E_SMOKE and trigger rollback
    world.policy.governance.smoke = switchyard::policy::types::SmokePolicy::Require {
        auto_rollback: true,
    };
}

#[then(regex = r"^the minimal smoke suite runs after apply$")]
pub async fn then_minimal_smoke_runs(world: &mut World) {
    // Presence of an apply.result summary implies post-apply flow executed
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && ev.get("action_id").is_none()
        {
            ok = true;
            break;
        }
    }
    assert!(
        ok,
        "expected apply.result summary after apply (smoke executed)"
    );
}

#[then(regex = r"^apply fails with error_id=E_SMOKE and exit_code=80$")]
pub async fn then_apply_fails_smoke(world: &mut World) {
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_SMOKE")
            && ev.get("exit_code").and_then(|v| v.as_i64()) == Some(80)
        {
            ok = true;
            break;
        }
    }
    assert!(ok, "expected E_SMOKE with exit_code=80");
}

#[then(regex = r"^executed actions are rolled back automatically$")]
pub async fn then_executed_actions_rolled_back(world: &mut World) {
    then_auto_rollback_occurs(world).await;
}
