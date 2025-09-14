// Facade for API module; delegates to submodules under src/api/
//! API facade and orchestrators.
//!
//! Construct using the builder:
//!
//! ```rust
//! use switchyard::api::Switchyard;
//! use switchyard::logging::JsonlSink;
//! use switchyard::policy::Policy;
//!
//! let facts = JsonlSink::default();
//! let audit = JsonlSink::default();
//! let _api = Switchyard::builder(facts, audit, Policy::default()).build();
//! ```

use crate::adapters::{Attestor, LockManager, OwnershipOracle, SmokeTestRunner};
use crate::logging::audit::new_run_id;
use crate::logging::{AuditSink, FactsEmitter, StageLogger};
use crate::policy::Policy;
use crate::types::{ApplyMode, ApplyReport, Plan, PlanInput, PreflightReport};
use serde_json::json;

// Internal API submodules (idiomatic; directory module)
mod apply;
mod overrides;
mod builder;
pub mod errors;
mod plan;
mod preflight;
mod rollback;
pub use builder::ApiBuilder;
pub use overrides::Overrides;
/// DX alias for `ApiBuilder`.
pub type SwitchyardBuilder<E, A> = ApiBuilder<E, A>;

/// Trait for lock managers that can be debugged
pub trait DebugLockManager: LockManager + std::fmt::Debug {}
impl<T: LockManager + std::fmt::Debug> DebugLockManager for T {}

/// Trait for ownership oracles that can be debugged
pub trait DebugOwnershipOracle: OwnershipOracle + std::fmt::Debug {}
impl<T: OwnershipOracle + std::fmt::Debug> DebugOwnershipOracle for T {}

/// Trait for attestors that can be debugged
pub trait DebugAttestor: Attestor + std::fmt::Debug {}
impl<T: Attestor + std::fmt::Debug> DebugAttestor for T {}

/// Trait for smoke test runners that can be debugged
pub trait DebugSmokeTestRunner: SmokeTestRunner + std::fmt::Debug {}
impl<T: SmokeTestRunner + std::fmt::Debug> DebugSmokeTestRunner for T {}

#[derive(Debug)]
pub struct Switchyard<E: FactsEmitter, A: AuditSink> {
    facts: E,
    audit: A,
    policy: Policy,
    overrides: Overrides,
    lock: Option<Box<dyn DebugLockManager>>, // None in dev/test; required in production
    owner: Option<Box<dyn DebugOwnershipOracle>>, // for strict ownership gating
    attest: Option<Box<dyn DebugAttestor>>,  // for final summary attestation
    smoke: Option<Box<dyn DebugSmokeTestRunner>>, // for post-apply health verification
    lock_timeout_ms: u64,
}

impl<E: FactsEmitter, A: AuditSink> Switchyard<E, A> {
    /// Construct a `Switchyard` with defaults. This function delegates to the builder.
    ///
    /// This delegates to `ApiBuilder::new(facts, audit, policy).build()` to
    /// avoid duplicating initialization logic.
    pub fn new(facts: E, audit: A, policy: Policy) -> Self {
        ApiBuilder::new(facts, audit, policy).build()
    }

    /// Entrypoint for constructing via the builder (default construction path).
    pub fn builder(facts: E, audit: A, policy: Policy) -> ApiBuilder<E, A> {
        ApiBuilder::new(facts, audit, policy)
    }

    /// Configure via `ApiBuilder::with_lock_manager`.
    #[must_use]
    pub fn with_lock_manager(mut self, lock: Box<dyn DebugLockManager>) -> Self {
        self.lock = Some(lock);
        self
    }

    /// Configure per-instance overrides for simulations (tests/controlled scenarios).
    #[must_use]
    pub fn with_overrides(mut self, overrides: Overrides) -> Self {
        self.overrides = overrides;
        self
    }

    /// Access the current per-instance overrides.
    #[must_use]
    pub fn overrides(&self) -> &Overrides {
        &self.overrides
    }

    /// Configure via `ApiBuilder::with_ownership_oracle`.
    #[must_use]
    pub fn with_ownership_oracle(mut self, owner: Box<dyn DebugOwnershipOracle>) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Configure via `ApiBuilder::with_attestor`.
    #[must_use]
    pub fn with_attestor(mut self, attest: Box<dyn DebugAttestor>) -> Self {
        self.attest = Some(attest);
        self
    }

    /// Configure via `ApiBuilder::with_smoke_runner`.
    #[must_use]
    pub fn with_smoke_runner(mut self, smoke: Box<dyn DebugSmokeTestRunner>) -> Self {
        self.smoke = Some(smoke);
        self
    }

    /// Configure via `ApiBuilder::with_lock_timeout_ms`.
    #[must_use]
    pub const fn with_lock_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.lock_timeout_ms = timeout_ms;
        self
    }

    pub fn plan(&self, input: PlanInput) -> Plan {
        #[cfg(feature = "tracing")]
        let _span = tracing::info_span!("switchyard.plan").entered();
        plan::build(self, input)
    }

    /// Execute preflight analysis for a plan. Returns a report with policy evaluation results.
    ///
    /// # Errors
    ///
    /// Returns an `ApiError` if the preflight analysis fails.
    pub fn preflight(&self, plan: &Plan) -> Result<PreflightReport, errors::ApiError> {
        #[cfg(feature = "tracing")]
        let _span = tracing::info_span!("switchyard.preflight").entered();
        Ok(preflight::run(self, plan))
    }

    /// Apply a plan in the specified mode. Returns a report with execution results.
    ///
    /// # Errors
    ///
    /// Returns an `ApiError` if the plan application fails.
    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> Result<ApplyReport, errors::ApiError> {
        #[cfg(feature = "tracing")]
        let _span = tracing::info_span!("switchyard.apply", mode = ?mode).entered();
        let report = apply::run(self, plan, mode);
        if matches!(mode, ApplyMode::Commit) && !report.errors.is_empty() {
            let joined = report.errors.join("; ").to_lowercase();
            if joined.contains("lock") {
                return Err(errors::ApiError::LockingTimeout(
                    "lock manager required or acquisition failed".to_string(),
                ));
            }
        }
        Ok(report)
    }

    pub fn plan_rollback_of(&self, report: &ApplyReport) -> Plan {
        #[cfg(feature = "tracing")]
        let _span = tracing::info_span!("switchyard.plan_rollback").entered();
        // Emit a planning fact for rollback to satisfy visibility and tests
        let plan_like = format!("rollback:{}", report
            .plan_uuid
            .map(|u| u.to_string())
            .unwrap_or_else(|| "unknown".to_string()));
        let pid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, plan_like.as_bytes());
        let run_id = new_run_id();
        let tctx = crate::logging::audit::AuditCtx::new(
            &self.facts,
            pid.to_string(),
            run_id,
            crate::logging::redact::now_iso(),
            crate::logging::audit::AuditMode {
                dry_run: false,
                redact: false,
            },
        );
        StageLogger::new(&tctx)
            .rollback()
            .merge(&json!({
                "planning": true,
                "executed": report.executed.len(),
            }))
            .emit_success();
        rollback::inverse_with_policy(&self.policy, report)
    }

    /// Prune backup artifacts for a given target according to retention policy knobs.
    ///
    /// Emits a `prune.result` fact with details about counts and policy used.
    /// Prune backups for a target path according to policy retention settings.
    ///
    /// # Errors
    ///
    /// Returns an `ApiError` if the backup pruning fails.
    pub fn prune_backups(
        &self,
        target: &crate::types::safepath::SafePath,
    ) -> Result<crate::types::PruneResult, errors::ApiError> {
        #[cfg(feature = "tracing")]
        let _span = tracing::info_span!(
            "switchyard.prune_backups",
            path = %target.as_path().display(),
            tag = %self.policy.backup.tag
        )
        .entered();
        // Synthesize a stable plan-like ID for pruning based on target path and tag.
        let plan_like = format!(
            "prune:{}:{}",
            target.as_path().display(),
            self.policy.backup.tag
        );
        let pid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, plan_like.as_bytes());
        let run_id = new_run_id();
        let tctx = crate::logging::audit::AuditCtx::new(
            &self.facts,
            pid.to_string(),
            run_id,
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
                StageLogger::new(&tctx).prune_result().merge(&json!({
                    "path": target.as_path().display().to_string(),
                    "backup_tag": self.policy.backup.tag,
                    "retention_count_limit": count_limit,
                    "retention_age_limit_ms": age_limit.map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX)),
                    "pruned_count": res.pruned_count,
                    "retained_count": res.retained_count,
                })).emit_success();
                Ok(res)
            }
            Err(e) => {
                StageLogger::new(&tctx)
                    .prune_result()
                    .merge(&json!({
                        "path": target.as_path().display().to_string(),
                        "backup_tag": self.policy.backup.tag,
                        "error": e.to_string(),
                        "error_id": errors::id_str(errors::ErrorId::E_GENERIC),
                        "exit_code": errors::exit_code_for(errors::ErrorId::E_GENERIC),
                    }))
                    .emit_failure();
                Err(errors::ApiError::FilesystemError(e.to_string()))
            }
        }
    }
}
