use cucumber::{given, then, when};

use crate::bdd_world::World;
use switchyard::api::Switchyard;
use switchyard::logging::{AuditSink, FactsEmitter};

// Compile-time Send + Sync assertions
fn assert_send_sync<T: Send + Sync>() {}

#[given(regex = r"^the Switchyard core types$")]
pub async fn given_core_types(_world: &mut World) {
    // Switchyard with trait bounds
    assert_send_sync::<
        Switchyard<crate::bdd_support::CollectingEmitter, crate::bdd_support::CollectingAudit>,
    >();
    // SafePath is clone & data-only; ensure Send+Sync
    assert_send_sync::<switchyard::types::safepath::SafePath>();
}

#[then(regex = r"^they are Send \+ Sync for safe use across threads$")]
pub async fn then_core_types_send_sync(_world: &mut World) {
    // If this compiled, the assertion holds
}

#[given(regex = r"^two threads invoking apply\(\) concurrently$")]
pub async fn given_two_threads(world: &mut World) {
    // Reuse existing lock manager setup to ensure real mutual exclusion
    crate::steps::locks_steps::given_with_lock(world).await;
}

#[given(regex = r"^a LockManager is configured$")]
pub async fn given_lock_manager_configured(world: &mut World) {
    crate::steps::locks_steps::given_with_lock(world).await;
}

#[when(regex = r"^both apply\(\) calls run$")]
pub async fn when_both_apply_run(world: &mut World) {
    crate::steps::locks_steps::when_two_apply_overlap(world).await
}

#[then(regex = r"^only one mutator proceeds at a time under the lock$")]
pub async fn then_mutual_exclusion(world: &mut World) {
    // Evidence: at least one apply.attempt event should have lock_wait_ms > 0 (waited for the other)
    let mut saw_wait = false;
    let mut attempts = 0u32;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.attempt") {
            attempts += 1;
            if let Some(ms) = ev.get("lock_wait_ms").and_then(|v| v.as_u64()) {
                if ms > 0 {
                    saw_wait = true;
                }
            }
        }
    }
    assert!(attempts >= 2, "expected two apply.attempt events");
    assert!(
        saw_wait,
        "expected at least one apply.attempt with lock_wait_ms > 0, indicating mutual exclusion"
    );
}
