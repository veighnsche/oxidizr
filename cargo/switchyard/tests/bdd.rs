#![cfg(any())]
#![cfg_attr(
    not(feature = "bdd"),
    allow(unused_imports, unused_variables, dead_code)
)]

use cucumber::World as _; // bring trait into scope for World::cucumber()
use serde_json::Value;
use std::path::Path;
use switchyard::adapters::{DefaultSmokeRunner, FileLockManager};
use switchyard::adapters::LockGuard;
use switchyard::api::{DebugAttestor, Switchyard};
use switchyard::policy::types::{ExdevPolicy, LockingPolicy, SmokePolicy};
use switchyard::policy::Policy;
use switchyard::preflight::yaml as preflight_yaml;
use switchyard::types::plan::{ApplyMode, LinkRequest, PlanInput};
use switchyard::types::report::{ApplyReport, PreflightReport};

mod bdd_support;
use bdd_support::{util, CollectingAudit, CollectingEmitter};
use bdd_support::env::EnvGuard;
use bdd_support::{facts, schema};

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
    lock_path: Option<std::path::PathBuf>,
    facts_dry: Option<Vec<Value>>,
    facts_real: Option<Vec<Value>>,
    // Scoped guards to ensure no cross-scenario leakage
    env_guards: Vec<EnvGuard>,
    lock_guards: Vec<Box<dyn LockGuard>>,
}

impl World {
    fn ensure_root(&mut self) -> &Path {
        if self.root.is_none() {
            self.root = Some(tempfile::TempDir::new().expect("tempdir"));
        }
        self.root.as_ref().unwrap().path()
    }

    fn rebuild_api(&mut self) {
        let api = Switchyard::builder(self.facts.clone(), self.audit.clone(), self.policy.clone())
            .build();
        self.api = Some(api);
    }

    fn ensure_api(&mut self) {
        if self.api.is_none() {
            self.rebuild_api();
        }
    }

    fn clear_facts(&mut self) {
        self.facts.0.lock().unwrap().clear();
        self.audit.0.lock().unwrap().clear();
    }

    fn mk_symlink(&mut self, link: &str, dest: &str) {
        let root = self.ensure_root().to_path_buf();
        let l = util::under_root(&root, link);
        let d = util::under_root(&root, dest);
        if let Some(p) = l.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        if let Some(p) = d.parent() {
            let _ = std::fs::create_dir_all(p);
        }
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
        if let Some(p) = sp_src.as_path().parent() {
            let _ = std::fs::create_dir_all(p);
        }
        let _ = std::fs::write(sp_src.as_path(), b"new");
        let sp_tgt = util::sp(&root, link);
        let mut input = PlanInput::default();
        input.link.push(LinkRequest {
            source: sp_src,
            target: sp_tgt,
        });
        self.ensure_api();
        let plan = self.api.as_ref().unwrap().plan(input);
        self.plan = Some(plan);
    }

    /// Ensure there is at least a minimal single-action plan available.
    fn ensure_plan_min(&mut self) {
        if self.plan.is_some() {
            return;
        }
        let link = self
            .last_link
            .clone()
            .unwrap_or_else(|| "/usr/bin/ls".to_string());
        let src = self
            .last_src
            .clone()
            .unwrap_or_else(|| "providerB/ls".to_string());
        self.build_single_swap(&link, &src);
    }

    /// Snapshot all emitted facts so far.
    fn all_facts(&self) -> Vec<Value> {
        self.facts.0.lock().unwrap().clone()
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

// Step definitions were migrated to tests/steps/* and wired via tests/bdd_main.rs.
// This legacy file is disabled from compilation by the crate-level cfg(any()) above.

#[cfg(not(feature = "bdd"))]
fn main() {}

#[cfg(all(feature = "bdd", any()))]
#[tokio::main(flavor = "multi_thread")]
async fn main() {}