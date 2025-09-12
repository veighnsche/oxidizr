//! Apply stage: executes plan actions with atomic symlink swap, backup/restore, and rollback.
//!
//! Side-effects:
//! - Emits Minimal Facts v1 for `apply.attempt` and `apply.result` per action, plus a summary.
//! - Enforces locking policy and maps failures to `E_LOCKING` with bounded wait.
//! - Enforces policy gating (unless `override_preflight=true`).
//! - Optionally runs smoke tests post-apply and triggers auto-rollback on failures.
//! - Optionally emits an attestation bundle on success.

use std::time::Instant;

use base64::Engine;
use serde_json::json;

use crate::fs;
use crate::logging::ts_for_mode;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::ids::{action_id, plan_id};
use crate::types::{Action, ApplyMode, ApplyReport, Plan};
use log::Level;

use super::errors::{exit_code_for, ErrorId};
use crate::logging::audit::{
    emit_apply_attempt, emit_apply_result, emit_rollback_step, AuditCtx, AuditMode,
};
use crate::policy::gating;
mod audit_fields;
mod handlers;

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
    let tctx = AuditCtx::new(
        &api.facts as &dyn FactsEmitter,
        pid.to_string(),
        ts_now.clone(),
        AuditMode {
            dry_run: dry,
            redact: dry,
        },
    );

    // Locking (required by default in Commit): acquire process lock with bounded wait; emit telemetry via apply.attempt
    api.audit.log(Level::Info, "apply: starting");
    let mut lock_wait_ms: Option<u64> = None;
    let mut _lock_guard: Option<Box<dyn crate::adapters::lock::LockGuard>> = None;
    if let Some(mgr) = &api.lock {
        let lt0 = Instant::now();
        match mgr.acquire_process_lock(api.lock_timeout_ms) {
            Ok(guard) => {
                lock_wait_ms = Some(lt0.elapsed().as_millis() as u64);
                _lock_guard = Some(guard);
            }
            Err(e) => {
                lock_wait_ms = Some(lt0.elapsed().as_millis() as u64);
                emit_apply_attempt(
                    &tctx,
                    "failure",
                    json!({
                        "lock_wait_ms": lock_wait_ms,
                        "error": e.to_string(),
                        "error_id": "E_LOCKING",
                        "exit_code": 30
                    }),
                );
                // Stage parity: also emit a summary apply.result failure for locking errors
                emit_apply_result(
                    &tctx,
                    "failure",
                    json!({
                        "error_id": crate::api::errors::id_str(ErrorId::E_LOCKING),
                        "exit_code": exit_code_for(ErrorId::E_LOCKING),
                    }),
                );
                let duration_ms = t0.elapsed().as_millis() as u64;
                api.audit
                    .log(Level::Error, "apply: lock acquisition failed (E_LOCKING)");
                return ApplyReport {
                    executed,
                    duration_ms,
                    errors: vec![format!("lock: {}", e)],
                    plan_uuid: Some(pid),
                    rolled_back,
                    rollback_errors,
                };
            }
        }
    } else {
        if !dry {
            // Enforce by default unless explicitly allowed through policy, or when require_lock_manager is set.
            let must_fail = api.policy.require_lock_manager || !api.policy.allow_unlocked_commit;
            if must_fail {
                emit_apply_attempt(
                    &tctx,
                    "failure",
                    json!({
                        "error_id": "E_LOCKING",
                        "exit_code": 30,
                    }),
                );
                // Stage parity: also emit a summary apply.result failure for locking errors
                emit_apply_result(
                    &tctx,
                    "failure",
                    json!({
                        "error_id": crate::api::errors::id_str(ErrorId::E_LOCKING),
                        "exit_code": exit_code_for(ErrorId::E_LOCKING),
                    }),
                );
                let duration_ms = t0.elapsed().as_millis() as u64;
                return ApplyReport {
                    executed,
                    duration_ms,
                    errors: vec!["lock manager required in Commit mode".to_string()],
                    plan_uuid: Some(pid),
                    rolled_back,
                    rollback_errors,
                };
            } else {
                emit_apply_attempt(
                    &tctx,
                    "warn",
                    json!({
                        "no_lock_manager": true,
                    }),
                );
            }
        } else {
            emit_apply_attempt(
                &tctx,
                "warn",
                json!({
                    "no_lock_manager": true,
                }),
            );
        }
    }

    // Minimal Facts v1: apply attempt summary (include lock_wait_ms when present)
    emit_apply_attempt(
        &tctx,
        "success",
        json!({
            "lock_wait_ms": lock_wait_ms,
        }),
    );

    // Policy gating: refuse to proceed when preflight would STOP, unless override is set.
    if !api.policy.override_preflight && !dry {
        let gating_errors = gating::gating_errors(&api.policy, api.owner.as_deref(), plan);
        if !gating_errors.is_empty() {
            api.audit
                .log(Level::Warn, "apply: policy gating rejected plan (E_POLICY)");
            let ec = exit_code_for(ErrorId::E_POLICY);
            // Emit per-action failures with action_id for visibility
            for (idx, act) in plan.actions.iter().enumerate() {
                let aid = action_id(&pid, act, idx).to_string();
                let path = match act {
                    Action::EnsureSymlink { target, .. } => target.as_path().display().to_string(),
                    Action::RestoreFromBackup { target } => target.as_path().display().to_string(),
                };
                emit_apply_result(
                    &tctx,
                    "failure",
                    json!({
                        "action_id": aid,
                        "path": path,
                        "error_id": "E_POLICY",
                        "exit_code": ec,
                    }),
                );
            }
            emit_apply_result(
                &tctx,
                "failure",
                json!({
                    "error_id": "E_POLICY",
                    "exit_code": ec,
                }),
            );
            let duration_ms = t0.elapsed().as_millis() as u64;
            return ApplyReport {
                executed,
                duration_ms,
                errors: gating_errors,
                plan_uuid: Some(pid),
                rolled_back,
                rollback_errors,
            };
        }
    }

    for (idx, act) in plan.actions.iter().enumerate() {
        match act {
            Action::EnsureSymlink { .. } => {
                let (exec, err) = handlers::handle_ensure_symlink(api, &tctx, &pid, act, idx, dry);
                if let Some(e) = err {
                    errors.push(e);
                }
                if let Some(a) = exec {
                    executed.push(a);
                }
            }
            Action::RestoreFromBackup { .. } => {
                let (exec, err) = handlers::handle_restore(api, &tctx, &pid, act, idx, dry);
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
                for prev in executed.iter().rev() {
                    match prev {
                        Action::EnsureSymlink {
                            source: _source,
                            target,
                        } => {
                            match fs::restore_file(
                                &target.as_path(),
                                dry,
                                api.policy.force_restore_best_effort,
                                &api.policy.backup_tag,
                            ) {
                                Ok(()) => {
                                    emit_rollback_step(
                                        &tctx,
                                        "success",
                                        &target.as_path().display().to_string(),
                                    );
                                }
                                Err(e) => {
                                    rollback_errors.push(format!(
                                        "rollback restore {} failed: {}",
                                        target.as_path().display(),
                                        e
                                    ));
                                    emit_rollback_step(
                                        &tctx,
                                        "failure",
                                        &target.as_path().display().to_string(),
                                    );
                                }
                            }
                        }
                        Action::RestoreFromBackup { .. } => {
                            // No reliable inverse without prior state capture; record informational error.
                            rollback_errors.push(
                                "rollback of RestoreFromBackup not supported (no prior state)"
                                    .to_string(),
                            );
                            emit_rollback_step(&tctx, "failure", "");
                        }
                    }
                }
            }
            break;
        }
    }

    // Optional smoke tests post-apply (only in Commit mode)
    if errors.is_empty() && !dry {
        if let Some(smoke) = &api.smoke {
            if smoke.run(plan).is_err() {
                errors.push("smoke tests failed".to_string());
                if !api.policy.disable_auto_rollback {
                    rolled_back = true;
                    for prev in executed.iter().rev() {
                        match prev {
                            Action::EnsureSymlink { source: _s, target } => {
                                let _ = fs::restore_file(
                                    &target.as_path(),
                                    dry,
                                    api.policy.force_restore_best_effort,
                                    &api.policy.backup_tag,
                                )
                                .map_err(|e| {
                                    rollback_errors.push(format!(
                                        "rollback restore {} failed: {}",
                                        target.as_path().display(),
                                        e
                                    ))
                                });
                            }
                            Action::RestoreFromBackup { .. } => {
                                rollback_errors.push(
                                    "rollback of RestoreFromBackup not supported (no prior state)"
                                        .to_string(),
                                );
                            }
                        }
                    }
                }
            }
        } else {
            // H3: Missing smoke runner when required
            if api.policy.require_smoke_in_commit {
                errors.push("smoke runner missing".to_string());
                if !api.policy.disable_auto_rollback {
                    rolled_back = true;
                    for prev in executed.iter().rev() {
                        match prev {
                            Action::EnsureSymlink { source: _s, target } => {
                                let _ = fs::restore_file(
                                    &target.as_path(),
                                    dry,
                                    api.policy.force_restore_best_effort,
                                    &api.policy.backup_tag,
                                )
                                .map_err(|e| {
                                    rollback_errors.push(format!(
                                        "rollback restore {} failed: {}",
                                        target.as_path().display(),
                                        e
                                    ))
                                });
                            }
                            Action::RestoreFromBackup { .. } => {
                                rollback_errors.push(
                                    "rollback of RestoreFromBackup not supported (no prior state)"
                                        .to_string(),
                                );
                            }
                        }
                    }
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
        "lock_wait_ms": lock_wait_ms,
    });
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
            if let Ok(sig) = att.sign(&bundle) {
                let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig.0.clone());
                // Compute sha256 of bundle for bundle_hash
                let mut hasher = sha2::Sha256::new();
                use sha2::Digest as _;
                hasher.update(&bundle);
                let bundle_hash = hex::encode(hasher.finalize());
                let att_json = json!({
                    "sig_alg": att.algorithm(),
                    "signature": sig_b64,
                    "bundle_hash": bundle_hash,
                    "public_key_id": att.key_id(),
                });
                // Merge attestation into fields
                let obj = fields.as_object_mut().unwrap();
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
        }
    }
    emit_apply_result(&tctx, decision, fields);
    api.audit.log(Level::Info, "apply: finished");

    // Compute total duration
    let duration_ms = t0.elapsed().as_millis() as u64;
    ApplyReport {
        executed,
        duration_ms,
        errors,
        plan_uuid: Some(pid),
        rolled_back,
        rollback_errors,
    }
}
