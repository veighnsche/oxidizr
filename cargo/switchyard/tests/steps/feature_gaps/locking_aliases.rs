use cucumber::then;

use crate::bdd_world::World;

#[then(regex = r"^only one mutator proceeds at a time$")]
pub async fn then_only_one_mutator(world: &mut World) {
    crate::steps::thread_safety_steps::then_mutual_exclusion(world).await;
}

#[then(regex = r"^concurrent apply is UNSUPPORTED and a WARN fact is emitted$")]
pub async fn then_warn_unsupported(world: &mut World) {
    crate::steps::locks_steps::then_warn_no_lock(world).await;
}

#[then(regex = r"^apply\.attempt includes lock_wait_ms$")]
pub async fn then_apply_attempt_includes_lock_wait(world: &mut World) {
    crate::steps::locks_steps::then_lock_wait(world).await;
}
