#![cfg_attr(
    not(feature = "bdd"),
    allow(unused_imports, unused_variables, dead_code)
)]

use serde_json::Value;
use std::path::Path;
use switchyard::adapters::{DefaultSmokeRunner, FileLockManager};
use switchyard::api::Switchyard;
use switchyard::policy::types::SmokePolicy;
use switchyard::policy::Policy;
use switchyard::types::plan::{ApplyMode, LinkRequest, PlanInput};
use switchyard::types::report::{ApplyReport, PreflightReport};

use crate::bdd_support::env::EnvGuard;
use crate::bdd_support::{util, CollectingAudit, CollectingEmitter};

#[derive(Default, cucumber::World)]
pub struct World {
    pub(crate) root: Option<tempfile::TempDir>,
    pub(crate) api: Option<Switchyard<CollectingEmitter, CollectingAudit>>,
    pub(crate) policy: Policy,
    pub(crate) plan: Option<switchyard::types::Plan>,
    pub(crate) preflight: Option<PreflightReport>,
    pub(crate) apply_report: Option<ApplyReport>,
    pub(crate) facts: CollectingEmitter,
    pub(crate) audit: CollectingAudit,
    pub(crate) last_link: Option<String>,
    pub(crate) last_src: Option<String>,
    pub(crate) lock_path: Option<std::path::PathBuf>,
    pub(crate) facts_dry: Option<Vec<Value>>,
    pub(crate) facts_real: Option<Vec<Value>>,
    // Scoped guards to ensure no cross-scenario leakage
    pub(crate) env_guards: Vec<EnvGuard>,
    pub(crate) lock_guards: Vec<Box<dyn switchyard::adapters::LockGuard>>,
}

impl World {
    pub fn ensure_root(&mut self) -> &Path {
        if self.root.is_none() {
            self.root = Some(tempfile::TempDir::new().expect("tempdir"));
        }
        self.root.as_ref().unwrap().path()
    }

    pub fn rebuild_api(&mut self) {
        let api = Switchyard::builder(self.facts.clone(), self.audit.clone(), self.policy.clone())
            .build();
        self.api = Some(api);
    }

    pub fn ensure_api(&mut self) {
        if self.api.is_none() {
            self.rebuild_api();
        }
    }

    pub fn clear_facts(&mut self) {
        self.facts.0.lock().unwrap().clear();
        self.audit.0.lock().unwrap().clear();
    }

    pub fn mk_symlink(&mut self, link: &str, dest: &str) {
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

    pub fn build_single_swap(&mut self, link: &str, src: &str) {
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
    pub fn ensure_plan_min(&mut self) {
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
    pub fn all_facts(&self) -> Vec<Value> {
        self.facts.0.lock().unwrap().clone()
    }

    /// Run a full real commit with preflight ensuring per-action facts.
    pub fn run_real(&mut self) {
        // Ensure Commit runs produce per-action facts even without a LockManager.
        if self.lock_path.is_none() {
            self.policy.governance.allow_unlocked_commit = true;
            self.rebuild_api();
        } else {
            self.ensure_api();
        }
        self.ensure_plan_min();
        let plan = self.plan.as_ref().unwrap();
        let _ = self.api.as_ref().unwrap().preflight(plan).unwrap();
        let _ = self
            .api
            .as_ref()
            .unwrap()
            .apply(plan, ApplyMode::Commit)
            .unwrap();
    }

    /// Run dry-run then commit back-to-back for parity checks.
    pub fn run_both_modes(&mut self) {
        if self.lock_path.is_none() {
            self.policy.governance.allow_unlocked_commit = true;
            self.rebuild_api();
        } else {
            self.ensure_api();
        }
        self.ensure_plan_min();
        let plan = self.plan.as_ref().unwrap();
        let _ = self
            .api
            .as_ref()
            .unwrap()
            .apply(plan, ApplyMode::DryRun)
            .unwrap();
        let _ = self
            .api
            .as_ref()
            .unwrap()
            .apply(plan, ApplyMode::Commit)
            .unwrap();
    }

    pub fn run_preflight_capture(&mut self) {
        self.ensure_api();
        self.ensure_plan_min();
        let plan = self.plan.as_ref().unwrap();
        self.preflight = Some(self.api.as_ref().unwrap().preflight(plan).unwrap());
    }

    pub fn apply_current_plan_commit(&mut self) {
        if self.lock_path.is_none() {
            self.policy.governance.allow_unlocked_commit = true;
            self.rebuild_api();
        } else {
            self.ensure_api();
        }
        let plan = self.plan.as_ref().unwrap();
        self.apply_report = Some(
            self.api
                .as_ref()
                .unwrap()
                .apply(plan, ApplyMode::Commit)
                .unwrap(),
        );
    }

    pub fn run_dry_and_store(&mut self) {
        self.ensure_api();
        self.ensure_plan_min();
        self.clear_facts();
        let plan = self.plan.as_ref().unwrap();
        let _ = self
            .api
            .as_ref()
            .unwrap()
            .apply(plan, ApplyMode::DryRun)
            .unwrap();
        self.facts_dry = Some(self.facts.0.lock().unwrap().clone());
    }

    pub fn enable_smoke(&mut self) {
        self.policy.governance.smoke = SmokePolicy::Require {
            auto_rollback: true,
        };
        let api = Switchyard::builder(self.facts.clone(), self.audit.clone(), self.policy.clone())
            .with_smoke_runner(Box::new(DefaultSmokeRunner))
            .build();
        self.api = Some(api);
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
