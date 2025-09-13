// Facade for API module; delegates to submodules under src/api/

use crate::adapters::{Attestor, LockManager, OwnershipOracle, SmokeTestRunner};
use crate::constants::DEFAULT_LOCK_TIMEOUT_MS;
use crate::logging::{AuditSink, FactsEmitter, StageLogger};
use serde_json::json;
use uuid::Uuid;
use crate::policy::Policy;
use crate::types::{ApplyMode, ApplyReport, Plan, PlanInput, PreflightReport};

// Internal API submodules (idiomatic; directory module)
mod apply;
pub mod errors;
mod builder;
mod plan;
mod preflight;
mod rollback;

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
        plan::build(self, input)
    }

    pub fn preflight(&self, plan: &Plan) -> Result<PreflightReport, errors::ApiError> {
        Ok(preflight::run(self, plan))
    }

    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> Result<ApplyReport, errors::ApiError> {
        Ok(apply::run(self, plan, mode))
    }

    pub fn plan_rollback_of(&self, report: &ApplyReport) -> Plan {
        rollback::inverse_with_policy(&self.policy, report)
    }

    /// Prune backup artifacts for a given target according to retention policy knobs.
    ///
    /// Emits a `prune.result` fact with details about counts and policy used.
    pub fn prune_backups(
        &self,
        target: &crate::types::safepath::SafePath,
    ) -> Result<crate::types::PruneResult, errors::ApiError> {
        // Synthesize a stable plan-like ID for pruning based on target path and tag.
        let plan_like = format!(
            "prune:{}:{}",
            target.as_path().display(),
            self.policy.backup.tag
        );
        let pid = Uuid::new_v5(&Uuid::NAMESPACE_URL, plan_like.as_bytes());
        let tctx = crate::logging::audit::AuditCtx::new(
            &self.facts as &dyn FactsEmitter,
            pid.to_string(),
            crate::logging::redact::now_iso(),
            crate::logging::audit::AuditMode {
                dry_run: false,
                redact: false,
            },
        );

        let count_limit = self.policy.retention_count_limit;
        let age_limit = self.policy.retention_age_limit;
        match crate::fs::backup::prune::prune_backups(
            &target.as_path(),
            &self.policy.backup.tag,
            count_limit,
            age_limit,
        ) {
            Ok(res) => {
                StageLogger::new(&tctx).prune_result().merge(json!({
                    "path": target.as_path().display().to_string(),
                    "backup_tag": self.policy.backup.tag,
                    "retention_count_limit": count_limit,
                    "retention_age_limit_ms": age_limit.map(|d| d.as_millis() as u64),
                    "pruned_count": res.pruned_count,
                    "retained_count": res.retained_count,
                })).emit_success();
                Ok(res)
            }
            Err(e) => {
                StageLogger::new(&tctx).prune_result().merge(json!({
                    "path": target.as_path().display().to_string(),
                    "backup_tag": self.policy.backup.tag,
                    "error": e.to_string(),
                    "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
                    "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
                })).emit_failure();
                Err(errors::ApiError::FilesystemError(e.to_string()))
            }
        }
    }
}
