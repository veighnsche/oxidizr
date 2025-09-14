use cucumber::{given, when, then};

use crate::bdd_world::World;
use switchyard::adapters::{FileLockManager, LockManager};
use switchyard::api::Switchyard;
use switchyard::types::plan::ApplyMode;

#[given(regex = r"^a production deployment with a LockManager$")]
pub async fn given_with_lock(world: &mut World) {
    let lock_path = world.ensure_root().join("switchyard.lock");
    world.lock_path = Some(lock_path.clone());
    let api = Switchyard::builder(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    )
    .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
    .build();
    world.api = Some(api);
}

#[given(regex = r"^a Switchyard built with a LockManager$")]
pub async fn given_with_lock_alias(world: &mut World) { given_with_lock(world).await }

#[given(regex = r"^a development environment without a LockManager$")]
pub async fn given_without_lock(world: &mut World) {
    world.policy.governance.locking = switchyard::policy::types::LockingPolicy::Optional;
    world.policy.governance.allow_unlocked_commit = true;
    world.rebuild_api();
}

#[given(regex = r"^a Switchyard without a LockManager$")]
pub async fn given_without_lock_alias(world: &mut World) { given_without_lock(world).await }

#[when(regex = r"^two apply\(\) calls overlap in time$")]
pub async fn when_two_apply_overlap(world: &mut World) {
    world.ensure_plan_min();
    let plan = world.plan.as_ref().unwrap().clone();
    let plan1 = plan.clone();
    let plan2 = plan.clone();
    let lock_path = world
        .lock_path
        .clone()
        .unwrap_or_else(|| world.ensure_root().join("switchyard.lock"));
    let lock1 = lock_path.clone();
    let lock2 = lock_path.clone();
    let facts1 = world.facts.clone();
    let audit1 = world.audit.clone();
    let policy1 = world.policy.clone();
    let h1 = std::thread::spawn(move || {
        let api = Switchyard::builder(facts1.clone(), audit1.clone(), policy1.clone())
            .with_lock_manager(Box::new(FileLockManager::new(lock1)))
            .build();
        let _ = api.apply(&plan1, ApplyMode::Commit);
    });
    let facts2 = world.facts.clone();
    let audit2 = world.audit.clone();
    let policy2 = world.policy.clone();
    let h2 = std::thread::spawn(move || {
        let api = Switchyard::builder(facts2.clone(), audit2.clone(), policy2.clone())
            .with_lock_manager(Box::new(FileLockManager::new(lock2)))
            .build();
        let _ = api.apply(&plan2, ApplyMode::Commit);
    });
    let _ = h1.join();
    let _ = h2.join();
}

#[when(regex = r"^both apply\(\) are started in Commit mode$")]
pub async fn when_both_started(world: &mut World) { when_two_apply_overlap(world).await }

#[then(regex = r"^facts record lock_wait_ms when available$")]
pub async fn then_lock_wait(world: &mut World) {
    let any_with = world
        .all_facts()
        .into_iter()
        .any(|e| e.get("lock_wait_ms").is_some());
    assert!(any_with, "no fact had lock_wait_ms");
}

#[then(regex = r"^a WARN fact is emitted stating concurrent apply is unsupported$")]
pub async fn then_warn_no_lock(world: &mut World) {
    let mut saw = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.attempt")
            && ev.get("decision").and_then(|v| v.as_str()) == Some("warn")
            && (ev.get("no_lock_manager").is_some()
                || ev.get("lock_backend").and_then(|v| v.as_str()) == Some("none"))
        { saw = true; break; }
    }
    assert!(saw, "expected WARN apply.attempt for no lock manager");
}

#[given(regex = r"^another apply\(\) is already holding the lock$")]
pub async fn given_other_holds_lock(world: &mut World) {
    let lock_path = world
        .lock_path
        .clone()
        .unwrap_or_else(|| world.ensure_root().join("switchyard.lock"));
    let mgr = FileLockManager::new(lock_path);
    // Hold guard for the duration of the scenario by storing in World
    let guard = mgr.acquire_process_lock(10_000).expect("acquire lock");
    world.lock_guards.push(guard);
}

#[given(regex = r"^a LockManager configured with a short timeout$")]
pub async fn given_short_timeout(world: &mut World) {
    let lock_path = world.ensure_root().join("switchyard.lock");
    world.lock_path = Some(lock_path.clone());
    let api = Switchyard::builder(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    )
    .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
    .with_lock_timeout_ms(150)
    .build();
    world.api = Some(api);
}

#[then(regex = r"^the stage fails with error_id=E_LOCKING and exit_code=30$")]
pub async fn then_locking_failure(world: &mut World) {
    let mut saw = false;
    for ev in world.all_facts() {
        if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_LOCKING")
            && ev.get("exit_code").and_then(|v| v.as_i64()) == Some(30)
        { saw = true; break; }
    }
    assert!(saw, "expected E_LOCKING with exit_code=30");
}

#[given(regex = r"^a contended lock with retries$")]
pub async fn given_contended(world: &mut World) {
    // Hold the lock briefly to force retries
    let lock_path = world
        .lock_path
        .clone()
        .unwrap_or_else(|| world.ensure_root().join("switchyard.lock"));
    std::thread::spawn(move || {
        let mgr = FileLockManager::new(lock_path);
        let g = mgr.acquire_process_lock(500).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        drop(g);
    });
}

#[then(regex = r"^apply.attempt includes lock_attempts approximating retry count$")]
pub async fn then_lock_attempts(world: &mut World) {
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.attempt") {
            if let Some(n) = ev.get("lock_attempts").and_then(|v| v.as_u64()) {
                if n >= 2 { ok = true; break; }
            }
        }
    }
    assert!(ok, "expected lock_attempts >= 2");
}
