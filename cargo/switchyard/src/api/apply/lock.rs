use std::time::Instant;

use log::Level;
use serde_json::json;
use uuid::Uuid;

use crate::api::errors::ErrorId;
use crate::api::Switchyard;
use crate::constants::LOCK_POLL_MS;
use crate::logging::{AuditSink, FactsEmitter, StageLogger};
use crate::types::{ApplyMode, ApplyReport};

use super::util::lock_backend_label;

pub(crate) struct LockInfo {
    pub lock_backend: String,
    pub lock_wait_ms: Option<u64>,
    pub approx_attempts: u64,
    pub guard: Option<Box<dyn crate::adapters::lock::LockGuard>>,
    pub early_report: Option<ApplyReport>,
}

impl LockInfo {
    #[must_use]
    #[allow(dead_code, reason = "deferred cleanup")]
    pub(super) const fn with_lock_timeout_ms(self, _timeout_ms: u64) -> Self {
        self
    }
}

pub(crate) fn acquire<E: FactsEmitter, A: AuditSink>(
    api: &Switchyard<E, A>,
    t0: Instant,
    pid: Uuid,
    mode: ApplyMode,
    tctx: &crate::logging::audit::AuditCtx<'_>,
) -> LockInfo {
    let dry = matches!(mode, ApplyMode::DryRun);
    let lock_backend = lock_backend_label(api.lock.as_deref());

    if let Some(_mgr) = &api.lock {
        let outcome = LockOrchestrator::acquire(api, mode);
        if let Some(g) = outcome.guard {
            return LockInfo {
                lock_backend,
                lock_wait_ms: outcome.lock_wait_ms,
                approx_attempts: outcome.approx_attempts,
                guard: Some(g),
                early_report: None,
            };
        }
        // Emit attempt and result failures with E_LOCKING
        LockOrchestrator::emit_failure(
            &StageLogger::new(tctx),
            &lock_backend,
            outcome.lock_wait_ms,
            outcome.approx_attempts,
        );
        api.audit
            .log(Level::Error, "apply: lock acquisition failed (E_LOCKING)");
        let duration_ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
        return LockInfo {
            lock_backend,
            lock_wait_ms: outcome.lock_wait_ms,
            approx_attempts: outcome.approx_attempts,
            guard: None,
            early_report: Some(LockOrchestrator::early_report(
                pid,
                duration_ms,
                outcome.err_msg.unwrap_or_else(|| "lock failed".to_string()),
            )),
        };
    } else if !dry {
        // Enforce by default unless explicitly allowed through policy, or when require_lock_manager is set.
        let must_fail = matches!(
            api.policy.governance.locking,
            crate::policy::types::LockingPolicy::Required
        ) || !api.policy.governance.allow_unlocked_commit;
        if must_fail {
            LockOrchestrator::emit_failure(&StageLogger::new(tctx), "none", None, 0);
            let duration_ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
            return LockInfo {
                lock_backend: "none".to_string(),
                lock_wait_ms: None,
                approx_attempts: 0,
                guard: None,
                early_report: Some(ApplyReport {
                    executed: Vec::new(),
                    duration_ms,
                    errors: vec!["lock manager required in Commit mode".to_string()],
                    plan_uuid: Some(pid),
                    rolled_back: false,
                    rollback_errors: Vec::new(),
                }),
            };
        }
        // Optional + allowed unlocked: emit a WARN attempt to signal visibility, then proceed.
        if matches!(
            api.policy.governance.locking,
            crate::policy::types::LockingPolicy::Optional
        ) && api.policy.governance.allow_unlocked_commit
        {
            StageLogger::new(tctx)
                .apply_attempt()
                .merge(&json!({
                    "lock_backend": "none",
                    "no_lock_manager": true,
                    "lock_attempts": 0u64,
                }))
                .emit_warn();
        }
    }
    // Default when no lock used (DryRun or no manager and allowed)
    let approx_attempts = u64::from(api.lock.is_some());
    LockInfo {
        lock_backend,
        lock_wait_ms: None,
        approx_attempts,
        guard: None,
        early_report: None,
    }
}

/// Facade for lock acquisition bookkeeping and telemetry.
struct LockOrchestrator;

struct LockOutcome {
    lock_wait_ms: Option<u64>,
    approx_attempts: u64,
    guard: Option<Box<dyn crate::adapters::lock::LockGuard>>,
    err_msg: Option<String>,
}

impl LockOrchestrator {
    fn acquire<E: FactsEmitter, A: AuditSink>(
        api: &Switchyard<E, A>,
        mode: ApplyMode,
    ) -> LockOutcome {
        if let Some(mgr) = &api.lock {
            let lt0 = Instant::now();
            match mgr.acquire_process_lock(api.lock_timeout_ms) {
                Ok(g) => {
                    let lock_wait_ms =
                        Some(u64::try_from(lt0.elapsed().as_millis()).unwrap_or(u64::MAX));
                    let mut approx_attempts = lock_wait_ms.map_or(1, |ms| 1 + (ms / LOCK_POLL_MS));
                    if approx_attempts < 2 {
                        approx_attempts = 2;
                    }
                    LockOutcome {
                        lock_wait_ms,
                        approx_attempts,
                        guard: Some(g),
                        err_msg: None,
                    }
                }
                Err(e) => {
                    let lock_wait_ms =
                        Some(u64::try_from(lt0.elapsed().as_millis()).unwrap_or(u64::MAX));
                    let mut approx_attempts = lock_wait_ms.map_or(1, |ms| 1 + (ms / LOCK_POLL_MS));
                    if approx_attempts < 2 {
                        approx_attempts = 2;
                    }
                    LockOutcome {
                        lock_wait_ms,
                        approx_attempts,
                        guard: None,
                        err_msg: Some(format!("lock: {e}")),
                    }
                }
            }
        } else {
            // No lock manager. In DryRun this is allowed; otherwise policy may require lock.
            let dry = matches!(mode, ApplyMode::DryRun);
            if dry {
                LockOutcome {
                    lock_wait_ms: None,
                    approx_attempts: 0,
                    guard: None,
                    err_msg: None,
                }
            } else {
                LockOutcome {
                    lock_wait_ms: None,
                    approx_attempts: 0,
                    guard: None,
                    err_msg: Some("lock manager required in Commit mode".to_string()),
                }
            }
        }
    }

    fn emit_failure(slog: &StageLogger<'_>, backend: &str, wait_ms: Option<u64>, attempts: u64) {
        slog.apply_attempt()
            .merge(&json!({
                "lock_backend": backend,
                "lock_wait_ms": wait_ms,
                "lock_attempts": attempts,
            }))
            .error_id(ErrorId::E_LOCKING)
            .exit_code_for(ErrorId::E_LOCKING)
            .emit_failure();
        slog.apply_result()
            .merge(&json!({
                "lock_backend": backend,
                "lock_wait_ms": wait_ms,
                "summary_error_ids": ["E_LOCKING"],
            }))
            .perf(0, 0, 0)
            .error_id(ErrorId::E_LOCKING)
            .exit_code_for(ErrorId::E_LOCKING)
            .emit_failure();
        // Historical parity: also emit a minimal apply.result failure with just error_id/exit_code
        slog.apply_result()
            .error_id(ErrorId::E_LOCKING)
            .exit_code_for(ErrorId::E_LOCKING)
            .emit_failure();
    }

    fn early_report(pid: Uuid, duration_ms: u64, error_msg: String) -> ApplyReport {
        ApplyReport {
            executed: Vec::new(),
            duration_ms,
            errors: vec![error_msg],
            plan_uuid: Some(pid),
            rolled_back: false,
            rollback_errors: Vec::new(),
        }
    }
}
