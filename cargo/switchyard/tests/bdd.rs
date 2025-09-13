#![cfg(feature = "bdd")]

use cucumber::World as CukeWorld;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Mutex};
use switchyard::adapters::{DefaultSmokeRunner, FileLockManager};
use switchyard::api::Switchyard;
use switchyard::policy::types::{ExdevPolicy, LockingPolicy, SmokePolicy};
use switchyard::policy::Policy;
use switchyard::types::plan::{ApplyMode, LinkRequest, PlanInput};
use switchyard::types::report::{ApplyReport, PreflightReport};

mod bdd_support;
use bdd_support::{util, CollectingAudit, CollectingEmitter};

#[derive(Default, cucumber::World)]
pub struct World {
    root: Option<tempfile::TempDir>,
    api: Option<Switchyard<CollectingEmitter, CollectingAudit>>,
    policy: Policy,
    plan: Option<switchyard::types::Plan>,
    preflight: Option<PreflightReport>,
    apply_report: Option<ApplyReport>,
    facts: CollectingEmitter,
    audit: CollectingAudit,
    last_link: Option<String>,
    last_src: Option<String>,
}

impl World {
    fn ensure_root(&mut self) -> &Path {
        if self.root.is_none() {
            self.root = Some(tempfile::TempDir::new().expect("tempdir"));
        }
        self.root.as_ref().unwrap().path()
    }

    fn rebuild_api(&mut self) {
        let api = Switchyard::builder(self.facts.clone(), self.audit.clone(), self.policy.clone()).build();
        self.api = Some(api);
    }

    fn ensure_api(&mut self) {
        if self.api.is_none() {
            self.rebuild_api();
        }
    }

    fn mk_symlink(&mut self, link: &str, dest: &str) {
        let root = self.ensure_root().to_path_buf();
        let l = util::under_root(&root, link);
        let d = util::under_root(&root, dest);
        if let Some(p) = l.parent() { let _ = std::fs::create_dir_all(p); }
        if let Some(p) = d.parent() { let _ = std::fs::create_dir_all(p); }
        // Ensure dest exists
        let _ = std::fs::write(&d, b"payload");
        let _ = std::fs::remove_file(&l);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&d, &l).expect("symlink");
        self.last_link = Some(link.to_string());
        self.last_src = Some(dest.to_string());
    }

    fn build_single_swap(&mut self, link: &str, src: &str) {
        let root = self.ensure_root().to_path_buf();
        // ensure source exists
        let sp_src = util::sp(&root, src);
        if let Some(p) = sp_src.as_path().parent() { let _ = std::fs::create_dir_all(p); }
        let _ = std::fs::write(sp_src.as_path(), b"new");
        let sp_tgt = util::sp(&root, link);
        let mut input = PlanInput::default();
        input.link.push(LinkRequest { source: sp_src, target: sp_tgt });
        self.ensure_api();
        let plan = self.api.as_ref().unwrap().plan(input);
        self.plan = Some(plan);
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("root", &self.root.as_ref().map(|_| "tempdir"))
            .field("policy", &"Policy{..}")
            .field("plan", &self.plan.as_ref().map(|p| p.actions.len()))
            .finish()
    }
}

// Step definitions grouped under a module so cucumber can register inventory.
mod steps {
    use super::*;
    use cucumber::{given, then, when};

    // Given steps
    #[given(regex = r"^/.+ is a symlink to .+$")]
    async fn given_symlink(world: &mut World, step: &cucumber::gherkin::Step) {
        let text = step.value.as_ref().unwrap().to_string();
        // Example: "/usr/bin/ls is a symlink to providerA/ls"
        let mut parts = text.split_whitespace();
        let link = parts.next().unwrap();
        // skip "is a symlink to"
        let _ = parts.next(); let _ = parts.next(); let _ = parts.next();
        let dest = parts.next().unwrap();
        world.mk_symlink(link, dest);
    }

    #[given(regex = r"^a plan with at least one action$")]
    async fn given_plan_min(world: &mut World) {
        let link = world.last_link.clone().unwrap_or_else(|| "/usr/bin/ls".to_string());
        let src = world.last_src.clone().unwrap_or_else(|| "providerB/ls".to_string());
        world.build_single_swap(&link, &src);
    }

    #[given(regex = r"^the target and staging directories reside on different filesystems$")]
    async fn given_exdev_env(_world: &mut World) { std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1"); }

    #[given(regex = r"^a production deployment with a LockManager$")]
    async fn given_with_lock(world: &mut World) {
        let lock_path = world.ensure_root().join("switchyard.lock");
        let api = Switchyard::builder(world.facts.clone(), world.audit.clone(), world.policy.clone())
            .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
            .build();
        world.api = Some(api);
    }

    #[given(regex = r"^a development environment without a LockManager$")]
    async fn given_without_lock(world: &mut World) {
        world.policy.governance.locking = LockingPolicy::Optional;
        world.policy.governance.allow_unlocked_commit = true;
        world.rebuild_api();
    }

    #[given(regex = r"^the minimal post-apply smoke suite is configured$")]
    async fn given_smoke(world: &mut World) {
        world.policy.governance.smoke = SmokePolicy::Require { auto_rollback: true };
        let api = Switchyard::builder(world.facts.clone(), world.audit.clone(), world.policy.clone())
            .with_smoke_runner(Box::new(DefaultSmokeRunner::default()))
            .build();
        world.api = Some(api);
    }

    // When steps
    #[when(regex = r"^I plan a swap to (\S+)$")]
    async fn when_plan_swap(world: &mut World, provider: String) {
        let link = world.last_link.clone().unwrap_or_else(|| "/usr/bin/ls".to_string());
        let src = format!("{}/ls", provider);
        world.build_single_swap(&link, &src);
    }

    #[when(regex = r"^I run preflight$")]
    async fn when_preflight(world: &mut World) { world.ensure_api(); let plan = world.plan.as_ref().unwrap(); world.preflight = Some(world.api.as_ref().unwrap().preflight(plan).unwrap()); }

    #[when(regex = r"^I apply the plan$")]
    async fn when_apply(world: &mut World) { world.ensure_api(); let plan = world.plan.as_ref().unwrap(); world.apply_report = Some(world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap()); }

    #[when(regex = r"^I run in dry-run mode$")]
    async fn when_run_dry(world: &mut World) { world.ensure_api(); let plan = world.plan.as_ref().unwrap(); let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::DryRun).unwrap(); }

    // Then steps (assertions on facts)
    fn all_facts(world: &World) -> Vec<Value> { world.facts.0.lock().unwrap().clone() }

    #[then(regex = r"^each fact carries schema_version=2$")]
    async fn then_schema_v2(world: &mut World) {
        for ev in all_facts(world) { assert_eq!(ev.get("schema_version").and_then(|v| v.as_i64()), Some(2)); }
    }

    #[then(regex = r"^facts record lock_wait_ms when available$")]
    async fn then_lock_wait(world: &mut World) {
        let any_with = all_facts(world).into_iter().any(|e| e.get("lock_wait_ms").is_some());
        assert!(any_with, "no fact had lock_wait_ms");
    }

    #[then(regex = r"^facts record degraded=true when policy allow_degraded_fs is enabled$")]
    async fn then_degraded_flag(world: &mut World) {
        // enable degraded and run apply to produce fact
        world.policy.apply.exdev = ExdevPolicy::DegradedFallback; world.rebuild_api();
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap();
        let mut saw = false;
        for ev in all_facts(world) {
            if let Some(d) = ev.get("degraded").and_then(|v| v.as_bool()) { if d { saw = true; break; } }
        }
        assert!(saw, "did not observe degraded=true fact");
    }

    #[then(regex = r"^the operation fails with error_id=E_EXDEV when allow_degraded_fs is disabled$")]
    async fn then_exdev_fail(world: &mut World) {
        world.policy.apply.exdev = ExdevPolicy::Fail; world.rebuild_api();
        std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1");
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap();
        let mut saw = false;
        for ev in all_facts(world) {
            if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_EXDEV") { saw = true; break; }
        }
        assert!(saw, "expected E_EXDEV in facts");
    }
}

#[tokio::test(flavor = "multi_thread")] 
async fn bdd_suite() {
    // Run all features under SPEC/features/
    World::cucumber()
        .fail_on_skipped()
        .run("SPEC/features")
        .await;
}
