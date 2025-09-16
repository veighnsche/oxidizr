use cucumber::{given, then, when};

use crate::bdd_world::World;
use switchyard::types::plan::ApplyMode;

#[given(regex = r"^failures during preflight or apply$")]
pub async fn given_failures_preflight_or_apply(world: &mut World) {
    crate::steps::apply_steps::given_target_fs_unsupported(world).await;
}

#[given(regex = r"^preflight STOP conditions are present$")]
pub async fn given_preflight_stop(world: &mut World) {
    crate::steps::apply_steps::given_target_fs_unsupported(world).await;
}

#[when(regex = r"^facts are emitted$")]
pub async fn when_facts_emitted(world: &mut World) {
    // Ensure we have some facts by running a minimal preflight
    crate::steps::preflight_steps::when_preflight(world).await;
}

#[then(regex = r"^error identifiers such as E_POLICY or E_LOCKING are stable strings$")]
pub async fn then_error_ids_stable(world: &mut World) {
    // Check that any error_id fields are string-typed
    for ev in world.all_facts() {
        if let Some(v) = ev.get("error_id") {
            assert!(v.is_string(), "error_id should be a string");
        }
    }
}

#[when(regex = r"^I compute the process exit$")]
pub async fn when_compute_process_exit(world: &mut World) {
    crate::steps::preflight_steps::when_preflight(world).await;
}

#[then(regex = r"^preflight summary carries error_id=E_POLICY and exit_code=10$")]
pub async fn then_preflight_summary_policy_10(world: &mut World) {
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("preflight.summary")
            && ev.get("error_id").and_then(|v| v.as_str()) == Some("E_POLICY")
            && ev.get("exit_code").and_then(|v| v.as_i64()) == Some(10)
        {
            ok = true;
            break;
        }
    }
    assert!(
        ok,
        "expected preflight.summary with error_id=E_POLICY and exit_code=10"
    );
}
