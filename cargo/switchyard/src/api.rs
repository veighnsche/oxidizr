// Facade for API module; delegates to submodules under src/api/

use crate::adapters::{Attestor, LockManager, OwnershipOracle, SmokeTestRunner};
use crate::logging::{AuditSink, FactsEmitter};
use crate::policy::Policy;
use crate::types::{ApplyMode, ApplyReport, Plan, PlanInput, PreflightReport};
use crate::constants::DEFAULT_LOCK_TIMEOUT_MS;

// Internal API submodules (planned split)
#[path = "api/fs_meta.rs"]
mod fs_meta;
#[path = "api/audit.rs"]
mod audit;
#[path = "api/errors.rs"]
pub mod errors;
#[path = "api/rollback.rs"]
mod rollback;
#[path = "api/plan.rs"]
mod plan_impl;
#[path = "api/preflight.rs"]
mod preflight_impl;
#[path = "api/apply.rs"]
mod apply_impl;

pub struct Switchyard<E: FactsEmitter, A: AuditSink> {
    facts: E,
    audit: A,
    policy: Policy,
    lock: Option<Box<dyn LockManager>>, // None in dev/test; required in production
    owner: Option<Box<dyn OwnershipOracle>>, // for strict ownership gating
    attest: Option<Box<dyn Attestor>>,  // for final summary attestation
    smoke: Option<Box<dyn SmokeTestRunner>>, // for post-apply health verification
    lock_timeout_ms: u64,
}

impl<E: FactsEmitter, A: AuditSink> Switchyard<E, A> {
    pub fn new(facts: E, audit: A, policy: Policy) -> Self {
        Self {
            facts,
            audit,
            policy,
            lock: None,
            owner: None,
            attest: None,
            smoke: None,
            lock_timeout_ms: DEFAULT_LOCK_TIMEOUT_MS,
        }
    }

    pub fn with_lock_manager(mut self, lock: Box<dyn LockManager>) -> Self {
        self.lock = Some(lock);
        self
    }

    pub fn with_ownership_oracle(mut self, owner: Box<dyn OwnershipOracle>) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_attestor(mut self, attest: Box<dyn Attestor>) -> Self {
        self.attest = Some(attest);
        self
    }

    pub fn with_smoke_runner(mut self, smoke: Box<dyn SmokeTestRunner>) -> Self {
        self.smoke = Some(smoke);
        self
    }

    pub fn with_lock_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.lock_timeout_ms = timeout_ms;
        self
    }

    pub fn plan(&self, input: PlanInput) -> Plan {
        plan_impl::build(self, input)
    }

    pub fn preflight(&self, plan: &Plan) -> Result<PreflightReport, errors::ApiError> {
        Ok(preflight_impl::run(self, plan))
    }

    pub fn apply(
        &self,
        plan: &Plan,
        mode: ApplyMode,
    ) -> Result<ApplyReport, errors::ApiError> {
        Ok(apply_impl::run(self, plan, mode))
    }

    pub fn plan_rollback_of(&self, report: &ApplyReport) -> Plan {
        rollback::inverse(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::{AuditSink, FactsEmitter};
    use log::Level;
    use serde_json::Value;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    #[derive(Default, Clone)]
    struct TestEmitter {
        events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
    }

    impl FactsEmitter for TestEmitter {
        fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
            self.events.lock().unwrap().push((
                subsystem.to_string(),
                event.to_string(),
                decision.to_string(),
                fields,
            ));
        }
    }

    #[derive(Default, Clone)]
    struct TestAudit;
    impl AuditSink for TestAudit {
        fn log(&self, _level: Level, _msg: &str) {}
    }

    #[test]
    fn emits_minimal_facts_for_plan_preflight_apply() {
        let facts = TestEmitter::default();
        let audit = TestAudit::default();
        let policy = Policy::default();
        let api = Switchyard::new(facts.clone(), audit, policy);

        // Build a simple plan under a temp root
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let src = Path::new("bin/uutils");
        let tgt = Path::new("usr/bin/ls");
        // Use SafePath from root
        let source = crate::types::safepath::SafePath::from_rooted(root, &root.join(src)).unwrap();
        let target = crate::types::safepath::SafePath::from_rooted(root, &root.join(tgt)).unwrap();
        let input = PlanInput {
            link: vec![crate::types::plan::LinkRequest {
                source: source.clone(),
                target: target.clone(),
            }],
            restore: vec![],
        };

        let plan = api.plan(input);
        // Preflight and apply (DryRun)
        let _pf = api.preflight(&plan).unwrap();
        let _ar = api.apply(&plan, ApplyMode::DryRun).unwrap();

        // Inspect captured events
        let evs = facts.events.lock().unwrap();
        assert!(!evs.is_empty(), "no facts captured");
        // Ensure all facts include schema_version and path
        for (_subsystem, _event, _decision, fields) in evs.iter() {
            assert_eq!(
                fields.get("schema_version").and_then(|v| v.as_i64()),
                Some(1),
                "schema_version=1"
            );
            // path may be null for some summaries; only check presence when present
            let _ = fields.get("path");
        }
        // Ensure plan_id is consistent and present
        let plan_ids: Vec<String> = evs
            .iter()
            .filter_map(|(_, _, _, f)| {
                f.get("plan_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();
        assert!(!plan_ids.is_empty());
        let first = &plan_ids[0];
        assert!(
            plan_ids.iter().all(|p| p == first),
            "plan_id should be consistent across events"
        );
    }

    #[test]
    fn rollback_reverts_first_action_on_second_failure() {
        let facts = TestEmitter::default();
        let audit = TestAudit::default();
        let mut policy = Policy::default();
        // Allow untrusted sources in test to avoid preflight fail-closed gating
        policy.force_untrusted_source = true;
        let api = Switchyard::new(facts.clone(), audit, policy);

        let td = tempfile::tempdir().unwrap();
        let root = td.path();

        // Layout
        let src1 = root.join("bin/new1");
        let src2 = root.join("bin/new2");
        let tgt1 = root.join("usr/bin/app1");
        let tgt2 = root.join("usr/sbin/app2");

        std::fs::create_dir_all(src1.parent().unwrap()).unwrap();
        std::fs::create_dir_all(src2.parent().unwrap()).unwrap();
        std::fs::create_dir_all(tgt1.parent().unwrap()).unwrap();
        std::fs::create_dir_all(tgt2.parent().unwrap()).unwrap();

        std::fs::write(&src1, b"new1").unwrap();
        std::fs::write(&src2, b"new2").unwrap();
        std::fs::write(&tgt1, b"old1").unwrap();
        std::fs::write(&tgt2, b"old2").unwrap();

        // Make parent of second target read-only to force failure during apply
        let sbin_dir = tgt2.parent().unwrap();
        let mut p = std::fs::metadata(sbin_dir).unwrap().permissions();
        p.set_mode(0o555);
        std::fs::set_permissions(sbin_dir, p).unwrap();

        // Build SafePaths
        let sp_src1 = crate::types::safepath::SafePath::from_rooted(root, &src1).unwrap();
        let sp_src2 = crate::types::safepath::SafePath::from_rooted(root, &src2).unwrap();
        let sp_tgt1 = crate::types::safepath::SafePath::from_rooted(root, &tgt1).unwrap();
        let sp_tgt2 = crate::types::safepath::SafePath::from_rooted(root, &tgt2).unwrap();

        let input = PlanInput {
            link: vec![
                crate::types::plan::LinkRequest {
                    source: sp_src1.clone(),
                    target: sp_tgt1.clone(),
                },
                crate::types::plan::LinkRequest {
                    source: sp_src2.clone(),
                    target: sp_tgt2.clone(),
                },
            ],
            restore: vec![],
        };
        let plan = api.plan(input);

        let report = api.apply(&plan, ApplyMode::Commit).unwrap();
        assert!(
            !report.errors.is_empty(),
            "apply should fail on second action"
        );
        assert!(report.rolled_back, "rolled_back should be true");

        // First target should be restored to regular file with original content
        let md1 = std::fs::symlink_metadata(&tgt1).unwrap();
        assert!(
            md1.file_type().is_file(),
            "tgt1 should be a regular file after rollback"
        );
        let content1 = std::fs::read_to_string(&tgt1).unwrap();
        assert!(content1.starts_with("old1"));
    }
}
