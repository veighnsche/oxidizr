use cucumber::{given, then, when};

use crate::bdd_world::World;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

fn ensure_dirs(p: &std::path::Path) {
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

#[given(regex = r"^a plan with three actions A, B, C where B will fail$")]
pub async fn given_three_actions_b_fails(world: &mut World) {
    // Ensure no EXDEV injection is active from other scenarios
    world
        .env_guards
        .push(crate::bdd_support::env::EnvGuard::new(
            "SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES",
            "0",
        ));
    world
        .env_guards
        .push(crate::bdd_support::env::EnvGuard::new(
            "SWITCHYARD_FORCE_EXDEV",
            "0",
        ));
    let root = world.ensure_root().to_path_buf();

    // Sources
    let s_a = root.join("new/A");
    let s_b = root.join("new/B");
    let s_c = root.join("new/C");
    ensure_dirs(&s_a);
    ensure_dirs(&s_b);
    ensure_dirs(&s_c);
    std::fs::write(&s_a, b"new-A").unwrap();
    std::fs::write(&s_b, b"new-B").unwrap();
    std::fs::write(&s_c, b"new-C").unwrap();

    // Targets: A and C are regular files; B is a directory to force unlink failure in executor
    let t_a = root.join("usr/bin/A");
    let t_b = root.join("usr/sbin/B");
    let t_c = root.join("usr/bin/C");
    ensure_dirs(&t_a);
    ensure_dirs(&t_b);
    ensure_dirs(&t_c);
    // Make A a regular file so the swap will snapshot and succeed
    std::fs::write(&t_a, b"old-A").unwrap();
    let _ = std::fs::create_dir_all(&t_b); // directory to trigger unlink failure
    // Keep B as a directory so unlink will fail during mutation
    std::fs::write(&t_c, b"old-C").unwrap();

    let sp_sa = SafePath::from_rooted(&root, &s_a).unwrap();
    let sp_sb = SafePath::from_rooted(&root, &s_b).unwrap();
    let sp_sc = SafePath::from_rooted(&root, &s_c).unwrap();
    let sp_ta = SafePath::from_rooted(&root, &t_a).unwrap();
    let sp_tb = SafePath::from_rooted(&root, &t_b).unwrap();
    let sp_tc = SafePath::from_rooted(&root, &t_c).unwrap();

    let plan = PlanInput {
        link: vec![
            LinkRequest {
                source: sp_sa,
                target: sp_ta,
            }, // A succeeds
            LinkRequest {
                source: sp_sb,
                target: sp_tb,
            }, // B fails
            LinkRequest {
                source: sp_sc,
                target: sp_tc,
            }, // C should not run
        ],
        restore: vec![],
    };
    world.policy.apply.override_preflight = true;
    world.policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;
    world.ensure_api();
    world.plan = Some(world.api.as_ref().unwrap().plan(plan));

    // Ensure apply won't be blocked
    world.policy.apply.override_preflight = true;
    world.policy.governance.allow_unlocked_commit = true;
    world.rebuild_api();
}

#[when(regex = r"^I apply the plan in Commit mode$")]
pub async fn when_apply_commit(world: &mut World) {
    // Use the world helper that stores ApplyReport for potential future assertions
    world.policy.apply.override_preflight = true;
    world.policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;
    world.apply_current_plan_commit();
}

#[then(regex = r"^the engine rolls back A in reverse order automatically$")]
pub async fn then_rollback_of_a(world: &mut World) {
    // Validate via rollback events: must include rollback of A and must not include rollback of C
    let root = world.ensure_root().to_path_buf();
    let path_a = root.join("usr/bin/A").display().to_string();
    let path_c = root.join("usr/bin/C").display().to_string();
    let mut saw_a = false;
    let mut saw_c = false;
    let mut dbg: Vec<String> = Vec::new();
    for ev in world.all_facts() {
        let stage = ev.get("stage").and_then(|v| v.as_str()).unwrap_or("");
        let path = ev.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let decision = ev.get("decision").and_then(|v| v.as_str()).unwrap_or("");
        let err = ev.get("error_id").and_then(|v| v.as_str()).unwrap_or("");
        let det = ev.get("error_detail").and_then(|v| v.as_str()).unwrap_or("");
        dbg.push(format!("{} {} {} {} {}", stage, decision, path, err, det));
        if stage == "rollback" {
            if ev.get("path").and_then(|v| v.as_str()) == Some(&path_a) { saw_a = true; }
            if ev.get("path").and_then(|v| v.as_str()) == Some(&path_c) { saw_c = true; }
        }
    }
    assert!(saw_a, "expected rollback of A to occur; events=\n{}", dbg.join("\n"));
    assert!(!saw_c, "did not expect rollback of C (should not have executed)");
}

#[then(regex = r"^emitted facts include partial restoration state if any rollback step fails$")]
pub async fn then_partial_restoration_if_any(world: &mut World) {
    // Best-effort: if any rollback emitted failure, ensure a rollback.summary event exists
    let mut any_rb_fail = false;
    let mut saw_summary = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("rollback")
            && ev.get("decision").and_then(|v| v.as_str()) == Some("failure")
        {
            any_rb_fail = true;
        }
        if ev.get("stage").and_then(|v| v.as_str()) == Some("rollback.summary") {
            saw_summary = true;
        }
    }
    if any_rb_fail {
        assert!(
            saw_summary,
            "expected rollback.summary when a rollback failed"
        );
    }
}

#[given(regex = r"^a plan that replaces a symlink then restores it$")]
pub async fn given_replace_then_restore(world: &mut World) {
    // Prepare initial symlink providerA -> app
    let root = world.ensure_root().to_path_buf();
    // providerA/app content
    let provider_a = root.join("providerA/app");
    ensure_dirs(&provider_a);
    std::fs::write(&provider_a, b"A").unwrap();
    // current link at usr/bin/app -> providerA/app
    world.mk_symlink("/usr/bin/app", "providerA/app");
    // Now plan to switch to providerB/app
    let provider_b = root.join("providerB/app");
    ensure_dirs(&provider_b);
    std::fs::write(&provider_b, b"B").unwrap();
    world.build_single_swap("/usr/bin/app", "providerB/app");
    // Ensure restore inversion has the prior snapshot to rely on during rollback plan
    world.policy.apply.capture_restore_snapshot = true;
    // require actual restore using captured snapshot (not best-effort)
    world.policy.apply.override_preflight = true;
    world.policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;
    world.policy.governance.allow_unlocked_commit = true;
    // use API without EXDEV override for normal swap/restore path
    world.rebuild_api();
}

#[when(regex = r"^I apply the plan and then apply a rollback plan twice$")]
pub async fn when_apply_and_rollback_twice(world: &mut World) {
    // Apply forward
    let plan = world.plan.as_ref().unwrap().clone();
    let report = world
        .api
        .as_ref()
        .unwrap()
        .apply(&plan, ApplyMode::Commit)
        .expect("apply ok");
    // Plan rollback and apply twice
    let rb1 = world.api.as_ref().unwrap().plan_rollback_of(&report);
    let _ = world.api.as_ref().unwrap().apply(&rb1, ApplyMode::Commit);
    let rb2 = world.api.as_ref().unwrap().plan_rollback_of(&report);
    let _ = world.api.as_ref().unwrap().apply(&rb2, ApplyMode::Commit);
}

#[then(regex = r"^the final link/file topology is identical to the prior state$")]
pub async fn then_topology_identical(world: &mut World) {
    // After forward then two rollbacks, expect /usr/bin/app resolves back to providerA/app
    let root = world.ensure_root().to_path_buf();
    let target = root.join("usr/bin/app");
    // If target is a symlink, resolve it
    let link_target = std::fs::read_link(&target).unwrap_or_else(|_| target.clone());
    let expected = root.join("providerA/app");
    assert_eq!(link_target, expected, "topology should match prior state");
}
