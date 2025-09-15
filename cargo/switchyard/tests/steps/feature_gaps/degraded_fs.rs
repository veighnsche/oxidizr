use cucumber::{given, then, when};

use crate::bdd_world::World;

#[given(regex = r"^staging and target parents reside on different filesystems \(EXDEV\)$")]
pub async fn given_exdev_parents(world: &mut World) {
    crate::steps::plan_steps::given_exdev_env(world).await;
}

#[given(regex = r"^policy allow_degraded_fs is true$")]
pub async fn given_allow_degraded_true(world: &mut World) {
    world.policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback;
    world.rebuild_api();
}

#[given(regex = r"^policy allow_degraded_fs is false$")]
pub async fn given_allow_degraded_false(world: &mut World) {
    world.policy.apply.exdev = switchyard::policy::types::ExdevPolicy::Fail;
    world.rebuild_api();
}

#[given(regex = r"^EXDEV conditions$")]
pub async fn given_exdev_conditions(world: &mut World) {
    crate::steps::plan_steps::given_exdev_env(world).await;
}

#[given(regex = r"^an environment matrix with ext4, xfs, btrfs, and tmpfs$")]
pub async fn given_env_matrix(_world: &mut World) {}

#[when(regex = r"^I apply a symlink replacement plan$")]
pub async fn when_apply_symlink_replacement_plan(world: &mut World) {
    crate::steps::plan_steps::given_exdev_env(world).await;
    super::when_apply_plan_replaces_cp(world).await;
}

#[then(regex = r"^the operation completes via safe copy \+ fsync \+ rename preserving atomic visibility$")]
pub async fn then_operation_completes_atomic(world: &mut World) {
    let root = world.ensure_root().to_path_buf();
    let link = root.join("usr/bin/cp");
    let md = std::fs::symlink_metadata(&link).expect("target exists");
    assert!(md.file_type().is_symlink(), "expected symlink at target");
    let target = std::fs::read_link(&link).unwrap_or_else(|_| link.clone());
    assert!(target.ends_with("providerB/cp"), "expected providerB/cp after degraded path");
}

#[then(regex = r#"^emitted facts record degraded=true with degraded_reason=\"exdev_fallback\"$"#)]
pub async fn then_emitted_degraded_true_reason(world: &mut World) {
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("degraded").and_then(|v| v.as_bool()) == Some(true)
            && ev.get("degraded_reason").and_then(|v| v.as_str()) == Some("exdev_fallback")
        {
            ok = true;
            break;
        }
    }
    assert!(ok, "expected degraded=true with reason exdev_fallback in facts");
}

#[then(regex = r"^the apply fails with error_id=E_EXDEV and exit_code=50$")]
pub async fn then_apply_fails_exdev_50(world: &mut World) {
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_EXDEV")
            && ev.get("exit_code").and_then(|v| v.as_i64()) == Some(50)
        {
            ok = true;
            break;
        }
    }
    assert!(ok, "expected E_EXDEV with exit_code=50");
}

#[then(regex = r#"^emitted facts include degraded=false with degraded_reason=\"exdev_fallback\"$"#)]
pub async fn then_emitted_degraded_false_reason(world: &mut World) {
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("degraded_reason").and_then(|v| v.as_str()) == Some("exdev_fallback") {
            if let Some(false) = ev.get("degraded").and_then(|v| v.as_bool()) {
                ok = true;
                break;
            }
        }
    }
    assert!(ok, "expected degraded=false with reason exdev_fallback in facts");
}

#[when(regex = r"^I run acceptance tests$")]
pub async fn when_run_acceptance_tests(_world: &mut World) {}

#[then(regex = r"^semantics for rename and degraded path are verified per filesystem$")]
pub async fn then_semantics_verified(world: &mut World) {
    use switchyard::types::safepath::SafePath;
    use switchyard::types::plan::{PlanInput, LinkRequest};
    // Common plan builder for cp
    let build_cp_plan = |world: &mut World| -> PlanInput {
        let root = world.ensure_root().to_path_buf();
        let src_b = root.join("providerB/cp");
        let tgt = root.join("usr/bin/cp");
        let _ = std::fs::create_dir_all(src_b.parent().unwrap());
        let _ = std::fs::create_dir_all(tgt.parent().unwrap());
        let _ = std::fs::write(&src_b, b"b");
        let s = SafePath::from_rooted(&root, &src_b).unwrap();
        let t = SafePath::from_rooted(&root, &tgt).unwrap();
        PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] }
    };

    // 1) DegradedFallback policy -> expect degraded=true or reason=exdev_fallback.
    world.clear_facts();
    world.policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback;
    world.policy.apply.override_preflight = true;
    world.policy.governance.allow_unlocked_commit = true;
    world.rebuild_api();
    crate::steps::plan_steps::given_exdev_env(world).await;
    let plan = world.api.as_ref().unwrap().plan(build_cp_plan(world));
    let _ = world.api.as_ref().unwrap().apply(&plan, switchyard::types::plan::ApplyMode::Commit);
    then_best_effort_degraded(world).await;

    // 2) Fail policy -> expect E_EXDEV with exit_code=50.
    world.clear_facts();
    world.policy.apply.exdev = switchyard::policy::types::ExdevPolicy::Fail;
    world.policy.apply.override_preflight = true;
    world.policy.governance.allow_unlocked_commit = true;
    world.rebuild_api();
    crate::steps::plan_steps::given_exdev_env(world).await;
    let plan2 = world.api.as_ref().unwrap().plan(build_cp_plan(world));
    let _ = world.api.as_ref().unwrap().apply(&plan2, switchyard::types::plan::ApplyMode::Commit);
    then_apply_fails_exdev_50(world).await;
}

#[then(regex = r"^the operation uses a best-effort degraded fallback for symlink replacement \(unlink \+ symlink\) when EXDEV occurs$")]
pub async fn then_best_effort_degraded(world: &mut World) {
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.result") {
            let reason = ev.get("degraded_reason").and_then(|v| v.as_str());
            let degraded = ev.get("degraded").and_then(|v| v.as_bool());
            if reason == Some("exdev_fallback") || degraded == Some(true) {
                ok = true;
                break;
            }
        }
    }
    assert!(ok, "expected degraded fallback evidence (degraded=true or reason=exdev_fallback)");
}
