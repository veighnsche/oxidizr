use std::time::Instant;

use log::Level;
use serde_json::json;
use uuid::Uuid;

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
    pub(super) const fn with_lock_timeout_ms(self, _timeout_ms: u64) -> Self {
        self
    }
}

pub(crate) fn acquire<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    t0: Instant,
    pid: Uuid,
    mode: ApplyMode,
    tctx: &crate::logging::audit::AuditCtx<'_>,
) -> LockInfo {
    let dry = matches!(mode, ApplyMode::DryRun);
    let mut lock_wait_ms: Option<u64> = None;
    let mut guard: Option<Box<dyn crate::adapters::lock::LockGuard>> = None;
    let lock_backend = lock_backend_label(api.lock.as_ref());

    if let Some(mgr) = &api.lock {
        let lt0 = Instant::now();
        match mgr.acquire_process_lock(api.lock_timeout_ms) {
            Ok(g) => {
                lock_wait_ms = Some(u64::try_from(lt0.elapsed().as_millis()).unwrap_or(u64::MAX));
                guard = Some(g);
            }
            Err(e) => {
                lock_wait_ms = Some(u64::try_from(lt0.elapsed().as_millis()).unwrap_or(u64::MAX));
                let approx_attempts = lock_wait_ms.map_or(1, |ms| 1 + (ms / LOCK_POLL_MS));
                StageLogger::new(tctx)
                    .apply_attempt()
                    .merge(&json!({
                        "lock_backend": lock_backend,
                        "lock_wait_ms": lock_wait_ms,
                        "lock_attempts": approx_attempts,
                        "error_id": "E_LOCKING",
                        "exit_code": 30,
                    }))
                    .emit_failure();
                StageLogger::new(tctx)
                    .apply_result()
                    .merge(&json!({
                        "lock_backend": lock_backend,
                        "lock_wait_ms": lock_wait_ms,
                        "perf": {"hash_ms": 0u64, "backup_ms": 0u64, "swap_ms": 0u64},
                        "error_id": "E_LOCKING",
                        "summary_error_ids": ["E_LOCKING"],
                        "exit_code": 30
                    }))
                    .emit_failure();
                // Stage parity: also emit a summary apply.result failure for locking errors
                StageLogger::new(tctx).apply_result().merge(&json!({
                    "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_LOCKING),
                    "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_LOCKING),
                })).emit_failure();
                api.audit
                    .log(Level::Error, "apply: lock acquisition failed (E_LOCKING)");
                let duration_ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
                return LockInfo {
                    lock_backend,
                    lock_wait_ms,
                    approx_attempts,
                    guard: None,
                    early_report: Some(ApplyReport {
                        executed: Vec::new(),
                        duration_ms,
                        errors: vec![format!("lock: {}", e)],
                        plan_uuid: Some(pid),
                        rolled_back: false,
                        rollback_errors: Vec::new(),
                    }),
                };
            }
        }
    } else if !dry {
        // Enforce by default unless explicitly allowed through policy, or when require_lock_manager is set.
        let must_fail = matches!(
            api.policy.governance.locking,
            crate::policy::types::LockingPolicy::Required
        ) || !api.policy.governance.allow_unlocked_commit;
        if must_fail {
            StageLogger::new(tctx)
                .apply_attempt()
                .merge(&json!({
                    "lock_backend": "none",
                    "lock_attempts": 0u64,
                    "error_id": "E_LOCKING",
                    "exit_code": 30,
                }))
                .emit_failure();
            // Stage parity: also emit a summary apply.result failure for locking errors
            StageLogger::new(tctx).apply_result().merge(&json!({
                "lock_backend": "none",
                "perf": {"hash_ms": 0u64, "backup_ms": 0u64, "swap_ms": 0u64},
                "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_LOCKING),
                "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_LOCKING),
            })).emit_failure();
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
    }

    let approx_attempts = lock_wait_ms.map_or_else(
        || u64::from(api.lock.is_some()),
        |ms| 1 + (ms / LOCK_POLL_MS),
    );
    LockInfo {
        lock_backend,
        lock_wait_ms,
        approx_attempts,
        guard,
        early_report: None,
    }
}
