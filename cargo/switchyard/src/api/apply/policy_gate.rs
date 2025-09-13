//! Policy gating helper for the Apply stage.
//!
//! Purpose: centralize enforcement of preflight gating at apply-time unless
//! override is set. Emits per-action `apply.result` failures and a summary
//! failure, preserving prior behavior and fields.
use serde_json::json;
use uuid::Uuid;

use crate::logging::{AuditSink, FactsEmitter, StageLogger};
use crate::types::{Action, ApplyReport, Plan};

use crate::api::errors::{exit_code_for, ErrorId};
use log::Level;

pub(crate) fn enforce<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    plan: &Plan,
    pid: Uuid,
    dry: bool,
    t0: std::time::Instant,
    slog: &StageLogger<'_>,
) -> Option<ApplyReport> {
    if api.policy.apply.override_preflight || dry {
        return None;
    }
    let gating_errors = crate::policy::gating::gating_errors(&api.policy, api.owner.as_deref(), plan);
    if gating_errors.is_empty() {
        return None;
    }
    // Parity: audit log at warn level when policy gating rejects
    api.audit
        .log(Level::Warn, "apply: policy gating rejected plan (E_POLICY)");
    // Emit per-action failures with action_id for visibility
    let ec = exit_code_for(ErrorId::E_POLICY);
    for (idx, act) in plan.actions.iter().enumerate() {
        let aid = crate::types::ids::action_id(&pid, act, idx).to_string();
        let path = match act {
            Action::EnsureSymlink { target, .. } => target.as_path().display().to_string(),
            Action::RestoreFromBackup { target } => target.as_path().display().to_string(),
        };
        slog.apply_result().merge(json!({
            "action_id": aid,
            "path": path,
            "error_id": "E_POLICY",
            "exit_code": ec,
        })).emit_failure();
    }
    slog.apply_result().merge(json!({
        "error_id": "E_POLICY",
        "exit_code": ec,
        "perf": {"hash_ms": 0u64, "backup_ms": 0u64, "swap_ms": 0u64},
    })).emit_failure();

    let duration_ms = t0.elapsed().as_millis() as u64;
    Some(ApplyReport {
        executed: Vec::new(),
        duration_ms,
        errors: gating_errors,
        plan_uuid: Some(pid),
        rolled_back: false,
        rollback_errors: Vec::new(),
    })
}
