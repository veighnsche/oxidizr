//! Apply stage: executes plan actions with atomic symlink swap, backup/restore, and rollback.
//!
//! Side-effects:
//! - Emits Audit v2 facts for `apply.attempt` and `apply.result` per action, plus a summary.
//! - Enforces locking policy and maps failures to `E_LOCKING` with bounded wait.
//! - Enforces policy gating (unless `override_preflight=true`).
//! - Optionally runs smoke tests post-apply and triggers auto-rollback on failures.
//! - Optionally emits an attestation bundle on success.

use std::time::Instant;

use crate::logging::audit::new_run_id;
use serde_json::json;

use crate::logging::ts_for_mode;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::ids::plan_id;
use crate::types::{Action, ApplyMode, ApplyReport, Plan};
use log::Level;

use super::errors::{exit_code_for, ErrorId};
use crate::logging::audit::{AuditCtx, AuditMode};
use crate::logging::StageLogger;
mod audit_fields;
mod handlers;
mod lock;
mod perf;
mod policy_gate;
mod rollback;
mod util;
use perf::PerfAgg;

// PerfAgg moved to perf.rs; lock backend helper and acquisition moved to util.rs and lock.rs

pub(crate) fn run<E: FactsEmitter, A: AuditSink>(
    api: &super::Switchyard<E, A>,
    plan: &Plan,
    mode: ApplyMode,
) -> ApplyReport {
    let t0 = Instant::now();
    let mut executed: Vec<Action> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut rollback_errors: Vec<String> = Vec::new();
    let mut rolled_back = false;
    let dry = matches!(mode, ApplyMode::DryRun);
    let pid = plan_id(plan);
    let ts_now = ts_for_mode(&mode);

    // Audit context
    let run_id = new_run_id();
    let tctx = AuditCtx::new(
        &api.facts,
        pid.to_string(),
        run_id,
        ts_now.clone(),
        AuditMode {
            dry_run: dry,
            redact: dry,
        },
    );
    let slog = StageLogger::new(&tctx);

    // Locking (required by default in Commit): acquire process lock with bounded wait; emit telemetry via apply.attempt
    api.audit.log(Level::Info, "apply: starting");
    let linfo = lock::acquire(api, t0, pid, mode, &tctx);
    let mut _lock_guard: Option<Box<dyn crate::adapters::lock::LockGuard>> = linfo.guard;
    if let Some(early) = linfo.early_report {
        return early;
    }

    // Audit v2: apply attempt summary (include lock_wait_ms when present)
    let approx_attempts = linfo.approx_attempts;
    slog.apply_attempt()
        .merge(&json!({
            "lock_backend": linfo.lock_backend,
            "lock_wait_ms": linfo.lock_wait_ms,
            "lock_attempts": approx_attempts,
        }))
        .emit_success();

    // Policy gating: refuse to proceed when preflight would STOP, unless override is set.
    if let Some(report) = policy_gate::enforce(api, plan, pid, dry, t0, &slog) {
        return report;
    }

    let mut perf_total = PerfAgg::default();
    for (idx, act) in plan.actions.iter().enumerate() {
        match act {
            Action::EnsureSymlink { .. } => {
                let (exec, err, perf) =
                    handlers::handle_ensure_symlink(api, &tctx, &pid, act, idx, dry, &slog);
                perf_total.hash_ms += perf.hash_ms;
                perf_total.backup_ms += perf.backup_ms;
                perf_total.swap_ms += perf.swap_ms;
                if let Some(e) = err {
                    errors.push(e);
                }
                if let Some(a) = exec {
                    executed.push(a);
                }
            }
            Action::RestoreFromBackup { .. } => {
                let (exec, err, perf) =
                    handlers::handle_restore(api, &tctx, &pid, act, idx, dry, &slog);
                perf_total.hash_ms += perf.hash_ms;
                perf_total.backup_ms += perf.backup_ms;
                perf_total.swap_ms += perf.swap_ms;
                if let Some(e) = err {
                    errors.push(e);
                }
                if let Some(a) = exec {
                    executed.push(a);
                }
            }
        }

        // On first failure, attempt reverse-order rollback for already executed actions.
        if !errors.is_empty() {
            if !dry {
                rolled_back = true;
                rollback::do_rollback(api, &executed, dry, &slog, &mut rollback_errors);
                rollback::emit_summary(&slog, &rollback_errors);
            }
            break;
        }
    }

    // Optional smoke tests post-apply (only in Commit mode)
    if errors.is_empty() && !dry {
        if let Some(smoke) = &api.smoke {
            if smoke.run(plan).is_err() {
                errors.push("smoke tests failed".to_string());
                let auto_rb = match api.policy.governance.smoke {
                    crate::policy::types::SmokePolicy::Require { auto_rollback } => auto_rollback,
                    crate::policy::types::SmokePolicy::Off => true,
                };
                if auto_rb {
                    rolled_back = true;
                    rollback::do_rollback(api, &executed, dry, &slog, &mut rollback_errors);
                }
            }
        } else {
            // H3: Missing smoke runner when required
            if matches!(
                api.policy.governance.smoke,
                crate::policy::types::SmokePolicy::Require { .. }
            ) {
                errors.push("smoke runner missing".to_string());
                let auto_rb = match api.policy.governance.smoke {
                    crate::policy::types::SmokePolicy::Require { auto_rollback } => auto_rollback,
                    crate::policy::types::SmokePolicy::Off => true,
                };
                if auto_rb {
                    rolled_back = true;
                    rollback::do_rollback(api, &executed, dry, &slog, &mut rollback_errors);
                }
            }
        }
    }

    // Final apply.result summary (after smoke tests/rollback)
    let decision = if errors.is_empty() {
        "success"
    } else {
        "failure"
    };
    let mut fields = json!({
        "lock_backend": linfo.lock_backend,
        "lock_wait_ms": linfo.lock_wait_ms,
    });
    // Attach perf aggregate (best-effort, zeros in DryRun may be redacted)
    if let Some(obj) = fields.as_object_mut() {
        obj.insert(
            "perf".to_string(),
            json!({
                "hash_ms": perf_total.hash_ms,
                "backup_ms": perf_total.backup_ms,
                "swap_ms": perf_total.swap_ms,
            }),
        );
    }
    // Optional attestation on success, non-dry-run
    if errors.is_empty() && !dry {
        if let Some(att) = &api.attest {
            // Construct a minimal attestation bundle with plan_id and executed actions count
            let bundle_json = json!({
                "plan_id": pid.to_string(),
                "executed": executed.len(),
                "rolled_back": rolled_back,
            });
            let bundle: Vec<u8> = serde_json::to_vec(&bundle_json).unwrap_or_default();
            if let Some(att_json) =
                crate::adapters::attest::build_attestation_fields(&**att, &bundle)
            {
                #[allow(
                    clippy::unwrap_used,
                    reason = "defer cleanup; will replace with safe shape normalizer later"
                )]
                let obj = fields
                    .as_object_mut()
                    .unwrap_or_else(|| panic!("Failed to get object mut reference"));
                obj.insert("attestation".to_string(), att_json);
            }
        }
    }

    // we already include ts/stage in helper
    // If we failed post-apply due to smoke, emit E_SMOKE at summary level; otherwise include a best-effort E_POLICY
    if decision == "failure" {
        if let Some(obj) = fields.as_object_mut() {
            if errors.iter().any(|e| e.contains("smoke")) {
                obj.insert(
                    "error_id".to_string(),
                    json!(crate::api::errors::id_str(ErrorId::E_SMOKE)),
                );
                obj.insert(
                    "exit_code".to_string(),
                    json!(exit_code_for(ErrorId::E_SMOKE)),
                );
            } else {
                // Default summary mapping for non-smoke failures
                obj.entry("error_id")
                    .or_insert(json!(crate::api::errors::id_str(ErrorId::E_POLICY)));
                obj.entry("exit_code")
                    .or_insert(json!(exit_code_for(ErrorId::E_POLICY)));
            }
            // Compute chain best-effort from collected error messages
            let chain = crate::api::errors::infer_summary_error_ids(&errors);
            obj.insert("summary_error_ids".to_string(), json!(chain));
        }
    }
    if decision == "failure" {
        slog.apply_result().merge(&fields).emit_failure();
    } else {
        slog.apply_result().merge(&fields).emit_success();
    }
    api.audit.log(Level::Info, "apply: finished");

    // Compute total duration
    let duration_ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
    ApplyReport {
        executed,
        duration_ms,
        errors,
        plan_uuid: Some(pid),
        rolled_back,
        rollback_errors,
    }
}
